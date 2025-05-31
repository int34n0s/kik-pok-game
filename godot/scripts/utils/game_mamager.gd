extends Node

var score = 0

@onready var score_label = $ScoreLabel
@onready var coins = %Coins

@onready var coins_number = coins.get_child_count()

func add_point():
	score += 1
	
	score_label.text = "You collected " + str(score) + "/" + str(coins_number) + " coins!"
