package main

import (
	"encoding/json"
)

const (
	maxPlayers = 100
)

type Hub struct {
	games map[string]ClientTuple
	connectionPool interface{}
	joinGame chan *Client // Spectators and players get the same Game information
	registerPlayer chan *Client // This requires a game id
	unregisterPlayer chan *Client // Player chooses to leave game
	//broadcast chan []byte
	sendToClient chan Request
	recieveFromClient chan ClientMessage
}

type Request {
	game string
	client string
	data []byte
}

type ClientMessage {
	client *Client
	message []byte
}

type ClientTuple struct {
	GameClient *Client
	PlayerClients map[string]*Client
}

func newHub() *Hub {
	return &Hub{
		games: make(map[string]ClientTuple),
		register: make(chan *Client),
		unregister: make(chan *Client),
		broadcast: make(chan []byte),
	}
}

func (h *Hub) run() {
	for {
		select {
		case client := <- h.register:
			if _, exists:= h.games[client.game]; exists {
				// Game exists, add
				h.games[client.game][client.id] = client

				// TODO: Alert other players in game of new player
			} else {
				// Game doesn't exist... Add it still...
				h.games[client.game] = map[string]*Client{client.id: client}

				// TODO: Return "Join with Code" option, and add to public games
			}
		case client := <- h.unregister:
			if clients, exists := h.games[client.game]; exists {
				if _, exists := clients[client.id]; exists {
					delete(h.games[client.game], client.id)
					close(client.send)
				}
			}
		//case message := <- h.broadcast:
		case request := <- sendToClient:
			/// GAME LOGIC!!!
			// Request format:
			/*
			{
				"vote": {},
				"pick": {},
				"send": {}
			}
			*/
			if clients, exists := h.games[request.game]; exists {
				if client, exists := clients[request.client]; exists {
					client.send <- request.data
				}
			}
		case message := <- recieveFromClient:

			gameOperation := GameOperation{}
			if err := json.Unmarshal(message.message, &gameOperation); err != nil{
				// Return Error response type
				message.Client.send([]byte("error?"))
			}
		}
	}
}

type Vote struct {
	CardId string
	Rating float64
}

type Pick struct {
	WinningPlayerId string
}

type Card struct {
	Id string,
	Ordinal int
}

type Send struct {
	PlayerId string
	CardSumbission *[] Card
	Wager []string
}

type Join struct {
	GameId string,
	PlayerName string
}

type GameOperation struct {
	Vote *Vote
	Pick *Pick
	Send *Send
	Join *Join
}

type ServerResponse struct {
	Error []string,
	Data []byte
}