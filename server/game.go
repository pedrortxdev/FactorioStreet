package main

import (
	"database/sql"
	"encoding/json"
	"log"
	"math/rand"
	"sync"
	"time"
)

type GameServer struct {
	db      *sql.DB
	prices  map[string]int64
	priceMu sync.RWMutex
	// Per-player throughput for offline sim
	throughput   map[int]float64 // userID -> money/second
	throughputMu sync.RWMutex
}

func newGameServer(db *sql.DB) *GameServer {
	gs := &GameServer{
		db: db,
		prices: map[string]int64{
			"iron_plate": 20, "copper_plate": 30, "coal_plate": 10,
			"silicon": 45, "glass": 15, "gold_ingot": 120, "uranium_cell": 300,
			"copper_wire": 50, "steel": 60, "plastic": 80,
			"circuit_board": 150, "processor": 600, "ai_core": 3500,
		},
		throughput: make(map[int]float64),
	}
	return gs
}

func (gs *GameServer) startMarketTicker(hub *Hub) {
	ticker := time.NewTicker(10 * time.Second)
	go func() {
		for range ticker.C {
			gs.tickMarket(hub)
		}
	}()
}

func (gs *GameServer) tickMarket(hub *Hub) {
	gs.priceMu.Lock()
	for item, price := range gs.prices {
		change := (rand.Float64()*0.2 - 0.1) // -10% to +10%
		newPrice := float64(price) * (1.0 + change)
		basePrice := gs.basePrice(item)
		if newPrice < float64(basePrice)*0.2 {
			newPrice = float64(basePrice) * 0.2
		}
		if newPrice > float64(basePrice)*4.0 {
			newPrice = float64(basePrice) * 4.0
		}
		gs.prices[item] = int64(newPrice)

		// Broadcast to all connected players
		evt := MarketEventData{Item: item, Price: int64(newPrice), PriceChange: change}
		data, _ := json.Marshal(evt)
		pkt, _ := json.Marshal(Packet{Type: PktMarketEvent, Data: data})

		hub.mu.RLock()
		for c := range hub.clients {
			select {
			case c.send <- pkt:
			default:
			}
		}
		hub.mu.RUnlock()
	}
	gs.priceMu.Unlock()
}

func (gs *GameServer) basePrice(item string) int64 {
	bases := map[string]int64{
		"iron_plate": 20, "copper_plate": 30, "coal_plate": 10,
		"silicon": 45, "glass": 15, "gold_ingot": 120, "uranium_cell": 300,
		"copper_wire": 50, "steel": 60, "plastic": 80,
		"circuit_board": 150, "processor": 600, "ai_core": 3500,
	}
	if p, ok := bases[item]; ok {
		return p
	}
	return 10
}

func (gs *GameServer) updatePlayerState(c *Client, sd *StateUpdateData) {
	// Update throughput tracking
	gs.throughputMu.Lock()
	gs.throughput[c.userID] = sd.ProductionRate
	gs.throughputMu.Unlock()

	// Persist to DB
	invJSON, _ := json.Marshal(sd.Inventory)
	gridJSON := string(sd.Grid)

	// CRITICAL: If grid is empty/null/[], do NOT overwrite the existing DB grid.
	// This happens during periodic syncs that only update money/x/y.
	if gridJSON == "" || gridJSON == "null" || gridJSON == "[]" || gridJSON == "{}" {
		_, err := gs.db.Exec(
			`INSERT INTO player_state (user_id, region_id, money, inventory, x, y, last_seen)
			 VALUES ($1, $2, $3, $4, $5, $6, NOW())
			 ON CONFLICT (user_id, region_id)
			 DO UPDATE SET money=$3, inventory=$4, x=$5, y=$6, last_seen=NOW()`,
			c.userID, c.region, sd.Money, string(invJSON), sd.X, sd.Y,
		)
		if err != nil {
			log.Printf("[GAME] Error saving partial state for %s: %v", c.username, err)
		}
	} else {
		_, err := gs.db.Exec(
			`INSERT INTO player_state (user_id, region_id, money, inventory, grid, x, y, last_seen)
			 VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
			 ON CONFLICT (user_id, region_id)
			 DO UPDATE SET money=$3, inventory=$4, grid=$5, x=$6, y=$7, last_seen=NOW()`,
			c.userID, c.region, sd.Money, string(invJSON), gridJSON, sd.X, sd.Y,
		)
		if err != nil {
			log.Printf("[GAME] Error saving full state for %s: %v", c.username, err)
		} else {
			log.Printf("[GAME] Grid salvo com sucesso para %s (%d bytes)", c.username, len(gridJSON))
		}
	}
}

func (gs *GameServer) playerDisconnected(c *Client) {
	// Save last_seen for offline sim
	gs.throughputMu.Lock()
	rate := gs.throughput[c.userID]
	delete(gs.throughput, c.userID)
	gs.throughputMu.Unlock()

	// Store rate for offline accumulation
	_, err := gs.db.Exec(
		`UPDATE player_state SET last_seen=NOW() WHERE user_id=$1 AND region_id=$2`,
		c.userID, c.region,
	)
	if err != nil {
		log.Printf("[GAME] Error updating last_seen for %s: %v", c.username, err)
	}

	log.Printf("[GAME] %s offline. Production rate was %.2f/s", c.username, rate)
}

// Called when a player reconnects — calculates offline earnings
func (gs *GameServer) calculateOfflineEarnings(userID int, region string) int64 {
	var money int64
	var lastSeen time.Time
	err := gs.db.QueryRow(
		`SELECT money, last_seen FROM player_state WHERE user_id=$1 AND region_id=$2`,
		userID, region,
	).Scan(&money, &lastSeen)
	if err != nil {
		return 0
	}

	elapsed := time.Since(lastSeen).Seconds()
	if elapsed < 60 {
		return 0 // No meaningful offline time
	}
	if elapsed > 86400 {
		elapsed = 86400 // Cap at 24h
	}

	// Estimate: base rate of 5 money/sec for each player (simplified)
	// A real system would store per-player throughput
	offlineEarnings := int64(elapsed * 5.0)
	log.Printf("[GAME] User %d was offline %.0fs, earned $%d", userID, elapsed, offlineEarnings)

	// Apply earnings
	gs.db.Exec(
		`UPDATE player_state SET money = money + $1, last_seen=NOW() WHERE user_id=$2 AND region_id=$3`,
		offlineEarnings, userID, region,
	)

	return offlineEarnings
}

func (gs *GameServer) getPlayerState(userID int, region string) (int64, map[string]int, []byte, float32, float32, int64, error) {
	var money int64
	var invJSON, gridJSON string
	var x, y float32
	var seed int64
	err := gs.db.QueryRow(
		`SELECT money, inventory, grid, x, y, seed FROM player_state WHERE user_id=$1 AND region_id=$2`,
		userID, region,
	).Scan(&money, &invJSON, &gridJSON, &x, &y, &seed)
	if err == sql.ErrNoRows {
		// New player: generate seeded random start
		seed = rand.Int63n(1000000000)
		gs.db.Exec(
			`INSERT INTO player_state (user_id, region_id, money, grid, x, y, seed) VALUES ($1, $2, 5000, '[]', 0, 0, $3)`,
			userID, region, seed,
		)
		return 5000, make(map[string]int), []byte("[]"), 0, 0, seed, nil
	}
	if err != nil {
		return 0, nil, nil, 0, 0, 0, err
	}

	inv := make(map[string]int)
	json.Unmarshal([]byte(invJSON), &inv)
	if gridJSON == "" || gridJSON == "{}" {
		gridJSON = "[]"
	}
	return money, inv, []byte(gridJSON), x, y, seed, nil
}

func (gs *GameServer) getPrices() map[string]int64 {
	gs.priceMu.RLock()
	defer gs.priceMu.RUnlock()
	cp := make(map[string]int64)
	for k, v := range gs.prices {
		cp[k] = v
	}
	return cp
}
