import Mathlib
import JuniperLean.JuniperJson

@[juniper_json]
theorem add_zero_custom (a: ℚ) : a + 0 = a := by
  exact Rat.add_zero a

@[juniper_json]
theorem add_comm_custom (a b: ℚ) : a + b = b + a := by
  exact Rat.add_comm a b

#save_juniper_json "../exported.json"
