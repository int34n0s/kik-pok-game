extends Node2D

const SPEED = 60

var direcation = 1

@onready var ray_cast_right = $RayCastRight
@onready var ray_cast_left = $RayCastLeft
@onready var animated_sprite = $AnimatedSprite

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	if ray_cast_right.is_colliding():
		direcation = -1
		animated_sprite.flip_h = true
	elif ray_cast_left.is_colliding():
		direcation = 1
		animated_sprite.flip_h = false
	
	position.x += direcation * SPEED * delta
