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
attribute [juniper_json] Rat.mul_inv_cancel
attribute [juniper_json] Rat.sub_eq_add_neg
attribute [juniper_json] Rat.inv_neg

@[juniper_json]
theorem involutive_neg_neg (a : ℚ) : -(-a) = a := by
  exact InvolutiveNeg.neg_neg a

@[juniper_json]
theorem involutive_inv_inv (a : ℚ) : a⁻¹⁻¹ = a := by
  exact InvolutiveInv.inv_inv a

#save_juniper_json "../exported.json"

#show_type_json Rat.inv_neg
