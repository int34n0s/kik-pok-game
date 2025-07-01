extends Node2D

func _input(event):
	if event is InputEventKey and event.pressed and event.keycode == KEY_1:
		get_tree().change_scene_to_file("res://scenes/testing/test_main.tscn")
		
	if event is InputEventKey and event.pressed and event.keycode == KEY_2:
		get_tree().change_scene_to_file("res://scenes/ui/login.tscn")
		
	if event is InputEventKey and event.pressed and event.keycode == KEY_3:
		get_tree().change_scene_to_file("res://scenes/world/entry.tscn")
