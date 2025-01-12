import Mathlib

open Lean Parser Meta Elab Command

-- thanks to Eric Weiser :)
deriving instance ToJson for Syntax.Preresolved
deriving instance ToJson for String.Pos
deriving instance ToJson for Substring
deriving instance ToJson for SourceInfo
deriving instance ToJson for Syntax
deriving instance ToJson for DataValue
deriving instance ToJson for Literal
deriving instance ToJson for LevelMVarId
deriving instance ToJson for Level
deriving instance ToJson for BinderInfo
instance : ToJson MData where
  toJson d := ToJson.toJson d.entries
deriving instance ToJson for Expr

-- some really thin boilerplate around `inferType`
elab "#show_type_json " t:term : command => Command.runTermElabM fun vars => do
  let e ← Term.elabTerm t none
  let typ ← inferType e
  logInfo m!"{ToJson.toJson typ}"

structure JuniperJsonEntry where
  name: Name
  type: Expr
deriving ToJson

initialize juniperJsonExtension : SimplePersistentEnvExtension JuniperJsonEntry (Array JuniperJsonEntry) ←
  registerSimplePersistentEnvExtension {
    name := `juniper_json
    addImportedFn := Array.flatMap id
    addEntryFn := Array.push
  }

initialize juniperJsonAttr : Unit ←
  registerBuiltinAttribute {
    name := `juniper_json
    descr := "adds a theorem to the rewrite conversion json file"
    applicationTime := AttributeApplicationTime.afterCompilation
    add := fun declName stx _attrKind => do
      let typ ← MetaM.run' do
        let typ ← inferType (Expr.const declName [])
        return typ
      let entry ← pure <| ⟨declName, typ⟩

      modifyEnv fun env =>
        juniperJsonExtension.addEntry env entry
    erase := fun _declName =>
      throwError "can't remove juniper_json (unimplemented)"
  }

def extractJuniperJson {m: Type → Type} [Monad m] [MonadEnv m] [MonadError m] :
    m (Array JuniperJsonEntry) := do
  return juniperJsonExtension.getState (← getEnv)

syntax (name := printJuniperJson) "#print_juniper_json" : command

elab_rules : command
| `(command| #print_juniper_json) => do
  let jj ← extractJuniperJson
  for entry in jj do
    println! ToJson.toJson entry

syntax (name := saveJuniperJson) "#save_juniper_json" term : command

@[command_elab saveJuniperJson] def saveJuniperJsonElab : CommandElab := fun stx => do
  if let some str := stx[1].isStrLit? then
    let jj ← extractJuniperJson
    let json: Json := ToJson.toJson jj
    let path := System.mkFilePath [str]
    let f := IO.FS.writeFile path json.pretty
    f
  else
    throwUnsupportedSyntax
