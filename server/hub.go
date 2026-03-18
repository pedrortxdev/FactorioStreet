package main

import (
	"encoding/json"
	"log"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

var _ = websocket.Upgrader{} // websocket imported for types

// --- Packet Types ---

type PacketType string

const (
	PktJoin        PacketType = "join"
	PktStateUpdate PacketType = "state_update"
	PktMarketEvent PacketType = "market_event"
	PktChat        PacketType = "chat"
	PktCursorSync  PacketType = "cursor_sync"
	PktRegionState PacketType = "region_state"
	PktPlayerList  PacketType = "player_list"
	PktError       PacketType = "error"
)

type Packet struct {
	Type   PacketType      `json:"type"`
	Data   json.RawMessage `json:"data"`
	Sender string          `json:"sender,omitempty"`
}

type JoinData struct {
	Region string `json:"region"`
}

type StateUpdateData struct {
	Tick           uint64          `json:"tick"`
	Money          int64           `json:"money"`
	Inventory      map[string]int  `json:"inventory"`
	ProductionRate float64         `json:"production_rate"`
	X              float32         `json:"x"`
	Y              float32         `json:"y"`
	Grid           json.RawMessage `json:"grid"`
}

type CursorData struct {
	X float32 `json:"x"`
	Y float32 `json:"y"`
}

type ChatData struct {
	Msg string `json:"msg"`
}

type MarketEventData struct {
	Item        string  `json:"item"`
	Price       int64   `json:"price"`
	PriceChange float64 `json:"price_change"`
}

type RegionStateData struct {
	Money           int64           `json:"money"`
	Inventory       map[string]int  `json:"inventory"`
	Prices          map[string]int64 `json:"prices"`
	OfflineEarnings int64           `json:"offline_earnings"`
	Region          string          `json:"region"`
	X               float32         `json:"x"`
	Y               float32         `json:"y"`
	Grid            json.RawMessage `json:"grid"`
	Seed            uint64          `json:"seed"`
}

// --- Client ---

type Client struct {
	hub      *Hub
	conn     *websocket.Conn
	send     chan []byte
	username string
	userID   int
	region   string
}

func (c *Client) readPump() {
	defer func() {
		c.hub.unregister <- c
		c.conn.Close()
	}()
	c.conn.SetReadLimit(10 * 1024 * 1024) // 10MB for 128x128 grid
	c.conn.SetReadDeadline(time.Now().Add(60 * time.Second))
	c.conn.SetPongHandler(func(string) error {
		c.conn.SetReadDeadline(time.Now().Add(60 * time.Second))
		return nil
	})
	for {
		_, message, err := c.conn.ReadMessage()
		if err != nil {
			break
		}
		var pkt Packet
		if err := json.Unmarshal(message, &pkt); err != nil {
			log.Printf("[WS] Error unmarshaling from %s: %v | RAW: %s", c.username, err, string(message))
			continue
		}
		pkt.Sender = c.username
		c.hub.handlePacket(c, &pkt)
	}
}

func (c *Client) writePump() {
	ticker := time.NewTicker(30 * time.Second)
	defer func() { ticker.Stop(); c.conn.Close() }()
	for {
		select {
		case msg, ok := <-c.send:
			c.conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if !ok {
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}
			if err := c.conn.WriteMessage(websocket.TextMessage, msg); err != nil {
				return
			}
		case <-ticker.C:
			c.conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

// --- Hub ---

type Hub struct {
	clients    map[*Client]bool
	register   chan *Client
	unregister chan *Client
	mu         sync.RWMutex
	game       *GameServer
	npcMgr     *NPCManager
}

func newHub(game *GameServer) *Hub {
	return &Hub{
		clients:    make(map[*Client]bool),
		register:   make(chan *Client),
		unregister: make(chan *Client),
		game:       game,
	}
}

func (h *Hub) run() {
	for {
		select {
		case client := <-h.register:
			h.mu.Lock()
			h.clients[client] = true
			h.mu.Unlock()
			log.Printf("[HUB] %s joined region %s (%d online)", client.username, client.region, h.countInRegion(client.region))
			h.broadcastPlayerList(client.region)

		case client := <-h.unregister:
			h.mu.Lock()
			if _, ok := h.clients[client]; ok {
				delete(h.clients, client)
				close(client.send)
			}
			h.mu.Unlock()
			log.Printf("[HUB] %s disconnected", client.username)
			// Trigger offline save
			h.game.playerDisconnected(client)
			h.broadcastPlayerList(client.region)
		}
	}
}

func (h *Hub) countInRegion(region string) int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	count := 0
	for c := range h.clients {
		if c.region == region {
			count++
		}
	}
	return count
}

func (h *Hub) broadcastToRegion(region string, data []byte, exclude *Client) {
	h.mu.RLock()
	defer h.mu.RUnlock()
	for c := range h.clients {
		if c.region == region && c != exclude {
			select {
			case c.send <- data:
			default:
			}
		}
	}
}

func (h *Hub) broadcastPlayerList(region string) {
	h.mu.RLock()
	var names []string
	for c := range h.clients {
		if c.region == region {
			names = append(names, c.username)
		}
	}
	h.mu.RUnlock()

	data, _ := json.Marshal(names)
	pkt, _ := json.Marshal(Packet{Type: PktPlayerList, Data: data})
	h.broadcastToRegion(region, pkt, nil)
}

func (h *Hub) handlePacket(c *Client, pkt *Packet) {
	log.Printf("[WS] Packet from %s: %s", c.username, pkt.Type)
	switch pkt.Type {
	case PktJoin:
		var jd JoinData
		json.Unmarshal(pkt.Data, &jd)
		c.region = jd.Region
		if c.region == "" {
			c.region = "rs_sul"
		}
		h.broadcastPlayerList(c.region)

	case PktStateUpdate:
		var sd StateUpdateData
		json.Unmarshal(pkt.Data, &sd)
		h.game.updatePlayerState(c, &sd)
		// Broadcast to other players in region so they see buildings/money changes
		pkt.Sender = c.username
		raw, _ := json.Marshal(pkt)
		h.broadcastToRegion(c.region, raw, c)

	case PktCursorSync:
		// Broadcast cursor to same region
		raw, _ := json.Marshal(pkt)
		h.broadcastToRegion(c.region, raw, c)

	case PktChat:
		raw, _ := json.Marshal(pkt)
		h.broadcastToRegion(c.region, raw, nil)
	}
}

func sendPacket(c *Client, pktType PacketType, data interface{}) {
	rawData, _ := json.Marshal(data)
	pkt, _ := json.Marshal(Packet{Type: pktType, Data: rawData})
	select {
	case c.send <- pkt:
	default:
	}
}
