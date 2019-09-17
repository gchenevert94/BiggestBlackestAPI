package biggestblackestapi

import (
	"context"
) // THIS CODE IS A STARTING POINT ONLY. IT WILL NOT BE UPDATED WITH SCHEMA CHANGES.

type Resolver struct{}

func (r *Resolver) Query() QueryResolver {
	return &queryResolver{r}
}

type queryResolver struct{ *Resolver }

func (r *queryResolver) GetSets(ctx context.Context) ([]*Set, error) {
	return []*Set{
		&Set{
			ID: "1",
			Name: "Box Expansion",
		},
	}, nil
}
func (r *queryResolver) GetCardsBySetName(ctx context.Context, sets []*string) ([]*Card, error) {
	return []*Card{
		&Card{
			ID: "1",
			IsBlack: false,
			Set: nil,
			Text: "Hello",
		},
	}, nil
}
func (r *queryResolver) GetCardsBySetID(ctx context.Context, sets []*string) ([]*Card, error) {
	return []*Card{
		&Card{
			ID: "1",
			IsBlack: false,
			Set: nil,
			Text: "Hello",
		},
	}, nil
}
