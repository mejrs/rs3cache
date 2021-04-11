from rs3cache.rs3cache import *
import functools

class Definitions:
	def __init__(self):
		pass

	@functools.cached_property
	def location_configs(self):
		return get_location_configs()

	@functools.cached_property
	def npc_configs(self):
		return get_npc_configs()

	@functools.cached_property
	def varbit_configs(self):
		return get_varbit_configs()
