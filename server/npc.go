package main

import (
	"encoding/json"
	"log"
	"math/rand"
	"sync"
	"time"
)

// NPC packet type
const PktNpcUpdate PacketType = "npc_update"

// ---- NPC Types ----

type NPCState string

const (
	NPCStateWander  NPCState = "wander"
	NPCStateInspect NPCState = "inspect"
	NPCStateReturn  NPCState = "return"
)

type NPCPlayer struct {
	ID       string   `json:"id"`
	Name     string   `json:"name"`
	PosX     float64  `json:"x"`
	PosY     float64  `json:"y"`
	State    NPCState `json:"state"`
	Region   string   `json:"region"`
	TargetX  float64  `json:"-"`
	TargetY  float64  `json:"-"`
	WaitFor  int      `json:"-"` // ticks to wait at current position
}

type NPCUpdateData struct {
	NPCs []NPCPlayer `json:"npcs"`
}

// ---- NPC Manager ----

type NPCManager struct {
	npcs   []*NPCPlayer
	mu     sync.RWMutex
	hub    *Hub
	gridSz float64
}

var npcNames = []string{
	"Agente Fiscal", "Inspetor Mendes", "Auditora Lima",
	"Delegado Costa", "Analista Braga", "Vizinha Nosy",
}

func newNPCManager(hub *Hub) *NPCManager {
	m := &NPCManager{
		hub:    hub,
		gridSz: 128.0, // GRID_SIZE
	}

	// Spawn NPCs for the default region
	regions := []string{"rs_sul", "sp_sudeste"}
	for i := 0; i < 3; i++ {
		region := regions[i%len(regions)]
		m.npcs = append(m.npcs, &NPCPlayer{
			ID:     generateID(),
			Name:   npcNames[i],
			PosX:   rand.Float64() * 128.0,
			PosY:   rand.Float64() * 128.0,
			State:  NPCStateWander,
			Region: region,
		})
	}

	return m
}

func generateID() string {
	const chars = "abcdefghijklmnopqrstuvwxyz0123456789"
	b := make([]byte, 8)
	for i := range b {
		b[i] = chars[rand.Intn(len(chars))]
	}
	return string(b)
}

func (m *NPCManager) Start() {
	ticker := time.NewTicker(2 * time.Second) // NPC tick every 2 seconds
	go func() {
		for range ticker.C {
			m.tick()
		}
	}()
}

func (m *NPCManager) tick() {
	m.mu.Lock()
	defer m.mu.Unlock()

	byRegion := make(map[string][]NPCPlayer)

	for _, npc := range m.npcs {
		if npc.WaitFor > 0 {
			npc.WaitFor--
		} else {
			m.moveNPC(npc)
		}
		byRegion[npc.Region] = append(byRegion[npc.Region], *npc)
	}

	// Broadcast per region
	for region, npcsInRegion := range byRegion {
		data, _ := json.Marshal(NPCUpdateData{NPCs: npcsInRegion})
		pkt, _ := json.Marshal(Packet{Type: PktNpcUpdate, Data: data})

		m.hub.mu.RLock()
		for c := range m.hub.clients {
			if c.region == region {
				select {
				case c.send <- pkt:
				default:
				}
			}
		}
		m.hub.mu.RUnlock()
	}
}

func (m *NPCManager) moveNPC(npc *NPCPlayer) {
	switch npc.State {
	case NPCStateWander:
		// Pick a random destination ~5-20 tiles away
		dx := (rand.Float64()*10 - 5)
		dy := (rand.Float64()*10 - 5)
		npc.TargetX = clamp(npc.PosX+dx, 0, m.gridSz-1)
		npc.TargetY = clamp(npc.PosY+dy, 0, m.gridSz-1)

		// Move towards target (instant for now, can lerp later)
		npc.PosX = npc.TargetX
		npc.PosY = npc.TargetY

		// 30% chance to switch to "inspect" mode
		if rand.Float64() < 0.3 {
			npc.State = NPCStateInspect
			npc.WaitFor = 5 // pause for 5 ticks (10 seconds)
			log.Printf("[NPC] %s is inspecting area at (%.1f, %.1f)", npc.Name, npc.PosX, npc.PosY)
		}

	case NPCStateInspect:
		// After inspecting, go back to wandering
		npc.State = NPCStateWander

	case NPCStateReturn:
		npc.State = NPCStateWander
	}
}

func clamp(v, min, max float64) float64 {
	if v < min {
		return min
	}
	if v > max {
		return max
	}
	return v
}
