package main

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
	"github.com/joho/godotenv"
	_ "github.com/lib/pq"
)

func getEnv(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func main() {
	// Load .env file
	_ = godotenv.Load()

	// Postgres
	dbURL := getEnv("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/factorio2?sslmode=disable")
	db, err := sql.Open("postgres", dbURL)
	if err != nil {
		log.Fatalf("Erro conectando Postgres: %v", err)
	}
	defer db.Close()
	if err := db.Ping(); err != nil {
		log.Fatalf("Postgres não responde: %v", err)
	}
	log.Println("[DB] Postgres conectado")

	// Auto-migrate: ensure new columns exist
	_, _ = db.Exec(`ALTER TABLE player_state ADD COLUMN IF NOT EXISTS x REAL DEFAULT 0`)
	_, _ = db.Exec(`ALTER TABLE player_state ADD COLUMN IF NOT EXISTS y REAL DEFAULT 0`)
	_, _ = db.Exec(`ALTER TABLE player_state ADD COLUMN IF NOT EXISTS grid TEXT DEFAULT '[]'`)
	_, _ = db.Exec(`ALTER TABLE player_state ADD COLUMN IF NOT EXISTS seed BIGINT DEFAULT floor(random() * 1000000000)`)
	
	// Migration: Fix existing '{}' grids to '[]' to avoid parsing errors
	_, _ = db.Exec(`UPDATE player_state SET grid = '[]' WHERE grid = '{}' OR grid = '' OR grid IS NULL`)
	
	log.Println("[DB] Migração e correção de sementes aplicadas")

	game := newGameServer(db)
	hub := newHub(game)
	go hub.run()
	game.startMarketTicker(hub)

	// Start NPC manager
	npcMgr := newNPCManager(hub)
	npcMgr.Start()
	hub.npcMgr = npcMgr

	r := gin.Default()

	// --- Public routes ---
	r.POST("/register", handleRegister(db))
	r.POST("/login", handleLogin(db))

	r.GET("/servers", func(c *gin.Context) {
		port := getEnv("GAME_PORT", "9000")
		servers := []gin.H{
			{"name": "Brasil Sul 1", "region": "rs_sul", "port": port, "players": hub.countInRegion("rs_sul"), "max": 100},
			{"name": "São Paulo Capital", "region": "sp_capital", "port": port, "players": hub.countInRegion("sp_capital"), "max": 100},
			{"name": "Minas Gerais", "region": "mg_gerais", "port": port, "players": hub.countInRegion("mg_gerais"), "max": 100},
		}
		c.JSON(http.StatusOK, servers)
	})

	r.GET("/prices", func(c *gin.Context) {
		c.JSON(http.StatusOK, game.getPrices())
	})

	// --- WebSocket endpoint ---
	r.GET("/ws", func(c *gin.Context) {
		tokenStr := c.Query("token")
		if tokenStr == "" {
			tokenStr = c.GetHeader("Authorization")
			if len(tokenStr) > 7 {
				tokenStr = tokenStr[7:]
			}
		}

		claims, valid := validateToken(tokenStr)
		if !valid {
			c.JSON(http.StatusUnauthorized, gin.H{"error": "Token inválido"})
			return
		}

		wsUpgrader := websocket.Upgrader{
			ReadBufferSize:  65536,
			WriteBufferSize: 65536,
			CheckOrigin:     func(r *http.Request) bool { return true },
		}

		conn, err := wsUpgrader.Upgrade(c.Writer, c.Request, nil)
		if err != nil {
			log.Printf("[WS] Upgrade failed: %v", err)
			return
		}

		region := c.Query("region")
		if region == "" {
			region = "rs_sul"
		}

		// Calculate offline earnings
		offlineEarnings := game.calculateOfflineEarnings(claims.UserID, region)

		// Load player state BEFORE building client
		money, inv, grid, x, y, seed, stateErr := game.getPlayerState(claims.UserID, region)
		if stateErr != nil {
			log.Printf("[WS] Error getting player state: %v", stateErr)
		}

		client := &Client{
			hub:      hub,
			conn:     conn,
			send:     make(chan []byte, 256),
			username: claims.Username,
			userID:   claims.UserID,
			region:   region,
		}

		// Send initial state
		if stateErr == nil {
			initData := RegionStateData{
				Money:           money,
				Inventory:       inv,
				Prices:          game.getPrices(),
				OfflineEarnings: offlineEarnings,
				Region:          region,
				X:               x,
				Y:               y,
				Grid:            json.RawMessage(grid),
				Seed:            uint64(seed),
			}
			rawData, _ := json.Marshal(initData)
			pkt, _ := json.Marshal(Packet{Type: PktRegionState, Data: rawData})
			select {
			case client.send <- pkt:
			default:
				log.Printf("[WS] Buffer cheio para %s", claims.Username)
			}
		}

		hub.register <- client
		go client.writePump()
		go client.readPump()
		log.Printf("[WS] %s conectado na região %s (offline earnings: $%d)", claims.Username, region, offlineEarnings)
	})

	port := getEnv("PORT", "8080")
	fmt.Printf("\n=== FACTORIO 2 SERVER ===\n")
	fmt.Printf("Auth + API: http://localhost:%s\n", port)
	fmt.Printf("WebSocket:  ws://localhost:%s/ws\n", port)
	fmt.Printf("=========================\n\n")
	r.Run(":" + port)
}
