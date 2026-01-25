extends CenterContainer

func _ready():
	pass

func _on_master_volume_changed(value: float):
	print("Master volume: %d" % int(value))
	# TODO: Apply audio bus volume

func _on_music_volume_changed(value: float):
	print("Music volume: %d" % int(value))
	# TODO: Apply audio bus volume

func _on_sfx_volume_changed(value: float):
	print("SFX volume: %d" % int(value))
	# TODO: Apply audio bus volume

func _on_back_pressed():
	get_tree().change_scene_to_file("res://scenes/ui/main_menu.tscn")
