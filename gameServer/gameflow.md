# HOW THIS WORK!?

| Client | Game Server | GraphQL |
|--------|-------------|---------|
| Create Game | Add To Mongo<br>Create new WebSocket Channel<br>Create game with config in MongoDB ||
| Join Game | Connect to WebSocket Channel<br>Update game in MongoDB ||
| Chosen | Randomly choose Card Czar and send on WebSocket Channel ||
| Display cards from WebSocket | Pull 7 * (#players) white cards + 1 black card<br>Send Cards over WebSocket | [Create Game](#new-round-query) $whiteCards = 7 * #players |
| Submit Card | Update game state with new submitted card ||
| Timer for Card Czar| WebSockets sends countdown for next round<br>Randomly select winner after timer and run next logic (query "best" card for choice) ||
| Player vote on card submissions || [Rate Card](#rate-card-combo) |
| Card Czar Choose Card | Run ***ROUND WON*** logic<br>Web Sockets send awarded Points (black cards)<br>Web Sockets choose next card czar<br> Web Sockets send new cards and choose black card | Draw next hand [New Round](#new-round-query) $whiteCars = #player-needed-cards (Cache in HashMap -> WebSocket) |
| Rince | And | Repeat |
| Player Leave| Pause Game | |
| Players can vote to remove player | Record votes ||
| Remove Player| WebSocket send Player Left (after timeout, or vote) and resume game ||
   
#####Game Object (mongodb)
```json
{
  "game-id": "GUID", // This could represent the "join with code"
  "remove-player-vote-total": 0,
  "game-state": 0, // Possible values Looking for Players, Playing, Finished
  "players":
    [
      {
        "id": "GUID",
        "player-state": 0, //Possible values Czar, Waiting, Played
        "score": 1,
        "white-cards": [
          {
            "id": "GUID"
          }
        ],
        "black-cards": [] // Black cards they've "won"
      }
    ],
  "deck-cursor-id": "GUID",
  "deck-shuffle-seed": "GUID",
  "decks-used": ["Deck-ID"],
  "card-czar-id": "GUID",
  "black-card-id": "GUID",
  "submitted-cards": [
    {
      "player-id": "GUID",
      "cards": [
        {
          "card-id": "GUID",
          "ordinal": 0
        }
      ],
      "wager": ["black-card-id"]
    }
  ]
}
```
  
#####new-round-query
```graphql
{
    czar: cards(color: BLACK, randomized: true, pagination:{pageSize: 1})
    players: cards(randomized: true, color: WHITE, pagination: {pageSize: $whiteCards})
}
```
#####rate-card-combo
```graphql
{
  mutation {
    rateCardCombination(rating: {whiteCard: $whiteCardId, blackCard: $blackCardId rating: $rating }) {
      result
    }
  }
}
```