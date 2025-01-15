import Mathlib
import JuniperLean.JuniperJson

attribute [juniper_json] Rat.add_zero
attribute [juniper_json] Rat.add_comm
attribute [juniper_json] Rat.add_assoc
attribute [juniper_json] Rat.add_mul
attribute [juniper_json] Rat.neg_add_cancel
attribute [juniper_json] Rat.mul_assoc
attribute [juniper_json] Rat.mul_comm
attribute [juniper_json] Rat.mul_one
attribute [juniper_json] Rat.mul_zero
attribute [juniper_json] Rat.mul_add
-- attribute [juniper_json] Rat.mul_inv_cancel -- Ne causes soundness issues (e.g. inv 0 != 0)
attribute [juniper_json] Rat.sub_eq_add_neg
attribute [juniper_json] Rat.inv_neg
attribute [juniper_json] Real.cos_add
attribute [juniper_json] Real.cos_neg
attribute [juniper_json] Real.cos_pi
attribute [juniper_json] Real.cos_sq
attribute [juniper_json] Real.cos_sq'
attribute [juniper_json] Real.cos_sub
attribute [juniper_json] Real.cos_zero
attribute [juniper_json] Real.cos_add_pi
attribute [juniper_json] Real.cos_add_pi_div_two
attribute [juniper_json] Real.cos_add_two_pi
attribute [juniper_json] Real.cos_pi_div_two
attribute [juniper_json] Real.cos_pi_div_three
attribute [juniper_json] Real.cos_pi_div_four
attribute [juniper_json] Real.cos_pi_div_six
attribute [juniper_json] Real.cos_pi_div_eight
attribute [juniper_json] Real.cos_pi_div_sixteen
attribute [juniper_json] Real.cos_pi_div_thirty_two
attribute [juniper_json] Real.cos_pi_div_two_sub
attribute [juniper_json] Real.cos_pi_sub
attribute [juniper_json] Real.cos_sq_add_sin_sq
attribute [juniper_json] Real.cos_sub_pi
attribute [juniper_json] Real.cos_sub_pi_div_two
attribute [juniper_json] Real.cos_sub_cos
attribute [juniper_json] Real.cos_sub_two_pi
attribute [juniper_json] Real.cos_three_mul
attribute [juniper_json] Real.cos_two_mul
attribute [juniper_json] Real.cos_two_pi
attribute [juniper_json] Real.cos_two_pi_sub
attribute [juniper_json] Real.sin_add
attribute [juniper_json] Real.sin_neg
attribute [juniper_json] Real.sin_pi
attribute [juniper_json] Real.sin_sq
attribute [juniper_json] Real.sin_sub
attribute [juniper_json] Real.sin_zero
attribute [juniper_json] Real.sin_add_pi
attribute [juniper_json] Real.sin_add_pi_div_two
attribute [juniper_json] Real.sin_add_two_pi
attribute [juniper_json] Real.sin_pi_div_two
attribute [juniper_json] Real.sin_pi_div_three
attribute [juniper_json] Real.sin_pi_div_four
attribute [juniper_json] Real.sin_pi_div_six
attribute [juniper_json] Real.sin_pi_div_eight
attribute [juniper_json] Real.sin_pi_div_sixteen
attribute [juniper_json] Real.sin_pi_div_thirty_two
attribute [juniper_json] Real.sin_pi_div_two_sub
attribute [juniper_json] Real.sin_pi_sub
attribute [juniper_json] Real.sin_sq_add_cos_sq
attribute [juniper_json] Real.sin_sq_eq_half_sub
attribute [juniper_json] Real.sin_sub_pi
attribute [juniper_json] Real.sin_sub_pi_div_two
attribute [juniper_json] Real.sin_sub_sin
attribute [juniper_json] Real.sin_sub_two_pi
attribute [juniper_json] Real.sin_three_mul
attribute [juniper_json] Real.sin_two_mul
attribute [juniper_json] Real.sin_two_pi
attribute [juniper_json] Real.sin_two_pi_sub
attribute [juniper_json] Real.two_mul_sin_mul_cos
attribute [juniper_json] Real.two_mul_sin_mul_sin
attribute [juniper_json] Real.one_rpow
attribute [juniper_json] Real.rpow_one
attribute [juniper_json] Real.rpow_zero

@[juniper_json]
theorem involutive_neg_neg (a : ℚ) : -(-a) = a := by
  exact InvolutiveNeg.neg_neg a

@[juniper_json]
theorem involutive_inv_inv (a : ℚ) : a⁻¹⁻¹ = a := by
  exact InvolutiveInv.inv_inv a

@[juniper_json]
theorem add_neg_neg (a b : ℚ) : (-a) + (-b) = -(a + b) := by
  linarith

@[juniper_json]
theorem mul_self (a : ℚ) : a * a = a ^ 2 := by
  linarith

#save_juniper_json "../exported.json"
