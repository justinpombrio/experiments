## Comptime

**Types**

    t ::= num
        | (t, t)
        | t → t
        | $t

    Γ ::= . | Γ,x:t

`$Γ` filters out all runtime variables from Γ:

    $(Γ,x:$t) = ($Γ),x:$t
    $(Γ,x:t)  = $Γ
    $ .       = .

Some types are equivalent to others:

          $$t === $t
    $(t0, t1) === ($t0, $t1)

**Expressions**

    e ::= <num>
        | (e, e) | e.0 | e.1
        | λx:t.e | e e | x
        | $e
    
    E ::= . | E,x=r

**Runtime Expressions**

    r ::= <num>
        | (r, r) | r.0 | r.1
        | λx:t.r | r r | x
        | [E, $λx:t.e]     -- comptime closure
    
    R ::= . | R,x=v

**Values**

    v ::= <num>
        | (v, v)
        | [R, λx:t.r]      -- runtime closure

**Type Checking**

         TNum ────────────────               TNum' ──────────────────
               Γ ⊢ <num> : num                      Γ ⊢ $<num> : $num
                                                                               
               Γ ⊢ e0 : t0                          Γ ⊢ $e0 : $t0
               Γ ⊢ e1 : t1                          Γ ⊢ $e1 : $t1
        TPair ────────────────────────      TPair' ──────────────────────────
               Γ ⊢ (e0, e1) : (t0, t1)              Γ ⊢ $(e0, e1) : $(t0, t1)

               Γ ⊢ e : (t0, t1)                     Γ ⊢ $e : $(t0, t1)
        TProj ─────────────────             TProj' ───────────────────
               Γ ⊢ e.0 : t0                         Γ ⊢ $e.0 : $t0
               Γ ⊢ e.1 : t1                         Γ ⊢ $e.1 : $t1    
                                                                               
               x:t ∈ Γ                              x:$t ∈ Γ
         TVar ──────────                     TVar' ────────────
               Γ ⊢ x : t                            Γ ⊢ $x : $t
                                                                               
               Γ,x:t ⊢ e: t'                        $Γ,x:t ⊢ e: t'
      TLambda ──────────────────────      TLambda' ──────────────────────────
               Γ ⊢ (λx:t.e) : t → t'                Γ ⊢ $(λx:t.e) : $(t → t')
                                                                               
               Γ ⊢ e_f : t → t'                     Γ ⊢ $e_f : $(t → t')
               Γ ⊢ e_arg : t                        Γ ⊢ $e_arg : $t
       TApply ───────────────────          Tapply' ───────────────────────
               Γ ⊢ e_f e_arg : t'                   Γ ⊢ $(e_f e_arg) : $t'

               Γ ⊢ $e : t
     TFlatten ────────────
               Γ ⊢ $$e : t

               Γ ⊢ e : $t
    TConstant ───────────
               Γ ⊢ e : t


**Compilation**

         CNum ──────────────────             CNum' ───────────────────
               E ⊢ <num> ↓ <num>                    E ⊢ $<num> ↓ <num>
                                                   
               E ⊢ e0 ↓ r0                          E ⊢ $e0 ↓ r0
               E ⊢ e1 ↓ r1                          E ⊢ $e1 ↓ r1
        CPair ────────────────────────      CPair' ─────────────────────────
               E ⊢ (e0, e1) ↓ (r0, r1)              E ⊢ $(e0, e1) ↓ (r0, r1)
                                        
               E ⊢ e ↓ r                            E ⊢ $e ↓ (r0, r1)
        CProj ──────────────                CProj' ──────────────────
               E ⊢ e.0 ↓ r.0                        E ⊢ $e.0 ↓ r0
               E ⊢ e.1 ↓ r.1                        E ⊢ $e.1 ↓ r1
                                        
               x=r ∈ E                              x=r ∈ E
         CVar ──────────                     CVar' ───────────
               E ⊢ x ↓ r                            E ⊢ $x ↓ r
                                        
               E,x=x ⊢ e ↓ r
      CLambda ────────────────────        CLambda' ─────────────────────────────
               E ⊢ λx:t.e ↓ λx:t.r                  E ⊢ $(λx:t.e) ↓ [E, $λx:t.e]
                                        
                                                    E ⊢ $e_f ↓ [E', $λx:t.e]
               E ⊢ e_f ↓ r_f                        E ⊢ $e_arg ↓ r_arg
               E ⊢ e_arg ↓ r_arg                    E',x=r_arg ⊢ e ↓ r
       CApply ──────────────────────────   CApply' ─────────────────────────
               E ⊢ e_f e_arg ↓ r_f r_arg            E ⊢ $(e_f e_arg) ↓ r
                                        
               E ⊢ $e ↓ r
     CFlatten ────────────
               E ⊢ $$e ↓ r

**Evaluation**

        RNum ──────────────────
              R ⊢ <num> ↓ <num>
             
             R ⊢ r0 ↓ v0
             R ⊢ r1 ↓ v1
      RPair ────────────────────────
             R ⊢ (r0, r1) ↓ (v0, v1)

             R ⊢ r ↓ (v0, v1)
      RProj ─────────────────
             R ⊢ r.0 ↓ v0
             R ⊢ r.1 ↓ v1

             x=v ∈ R
       RVar ──────────
             R ⊢ x ↓ v

    RLambda ─────────────────────────
             R ⊢ λx:t.r ↓ [R, λx:t.r]

             R ⊢ r_f ↓ [R', λx:t.r']
             R ⊢ r_arg ↓ v_arg
             R',x=v_arg ⊢ r' ↓ v
     RApply ────────────────────────
             R ⊢ r_f r_arg ↓ v
