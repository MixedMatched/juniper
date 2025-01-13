import Mathlib
import JuniperLean.JuniperJson

@[juniper_json]
theorem add_zero_custom (a: ℚ) : a + 0 = a := by
  exact Rat.add_zero a

@[juniper_json]
theorem add_comm_custom (a b: ℚ) : a + b = b + a := by
  exact Rat.add_comm a b

@[juniper_json]
theorem double_neg (a: ℚ) : -(-a) = a:= by
  exact InvolutiveNeg.neg_neg a

#save_juniper_json "../exported.json"

theorem sin_cos_one (a b: ℝ) : √ a = b := sorry

#show_type_json sin_cos_one


#show_type_json double_neg
