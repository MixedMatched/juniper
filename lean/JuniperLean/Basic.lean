import Mathlib

-- thanks to Eric Weiser :)
deriving instance Lean.ToJson for Lean.Syntax.Preresolved
deriving instance Lean.ToJson for String.Pos
deriving instance Lean.ToJson for Substring
deriving instance Lean.ToJson for Lean.SourceInfo
deriving instance Lean.ToJson for Lean.Syntax
deriving instance Lean.ToJson for Lean.DataValue
deriving instance Lean.ToJson for Lean.Literal
deriving instance Lean.ToJson for Lean.LevelMVarId
deriving instance Lean.ToJson for Lean.Level
deriving instance Lean.ToJson for Lean.BinderInfo
instance : Lean.ToJson Lean.MData where
  toJson d := Lean.ToJson.toJson d.entries
deriving instance Lean.ToJson for Lean.Expr

-- some really thin boilerplate around `inferType`
elab "#show_type_json " t:term : command => Lean.Elab.Command.runTermElabM fun vars => do
  let e ← Lean.Elab.Term.elabTerm t none
  let typ ← Lean.Meta.inferType e
  Lean.logInfo m!"{Lean.ToJson.toJson typ}"

-- your example
theorem zero_add_custom (a b c: ℚ) : a * 6 + b = 3 * (2 + c / 4) := by
  sorry

theorem add_comm_custom (a b: ℚ) : a + b = b + a := by
  exact Rat.add_comm a b

variable (a b c : ℚ)

def xxx : Prop := c ≠ 0

theorem test : xxx c → Rat.mul c a = b := sorry

#show_type_json zero_add_custom
