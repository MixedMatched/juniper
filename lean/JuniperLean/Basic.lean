import Mathlib
import JuniperLean.JuniperJson

@[juniper_json]
theorem zero_add_custom (a b: ℚ) : a * 6 + b = 3 * (2 + 1 / 4) := by
  sorry

@[juniper_json]
theorem add_comm_custom (a b: ℚ) : a + b = b + a := by
  exact Rat.add_comm a b

#save_juniper_json "../exported.json"
