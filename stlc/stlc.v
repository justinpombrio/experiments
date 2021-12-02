Require Import String.
Require Import Coq.Program.Equality.

Definition string_eq (x y: string) : bool :=
  match string_dec x y with
    | left _  => true
    | right _ => false
  end.

Inductive Tipe : Set :=
  | t_bool : Tipe
  | t_lamb : Tipe -> Tipe -> Tipe. (*arrow may be better term*)

Notation "A :-> B" := (t_lamb A B) (at level 70).

Inductive Term : Set :=
  | true  : Term
  | false : Term
  | ifte  : Term -> Term -> Term -> Term
  | var   : string -> Term
  | lamb  : string -> Term -> Term
  | app   : Term -> Term -> Term.

Inductive value : Term -> Prop :=
  | v_true  : value true
  | v_false : value false
  | v_lamb  : forall x b, value (lamb x b).

Record Val :=
  { t :> Term;
    v : value t }.
  


Inductive Ctx : Set := (*big gamma*)
  | empty : Ctx
  | cons  : Ctx -> string -> Tipe -> Ctx.

Fixpoint ctx_lookup (G: Ctx) (x: string) : option Tipe :=
  match G with
    | empty => None
    | cons G y A =>
      if string_eq y x then Some(A) else ctx_lookup G x
  end.

Inductive Subs : Set := (*little gamma*)
  | empty_subs : Subs
  | cons_subs  : Subs -> string -> Term -> Subs.

Fixpoint subs_lookup (g: Subs) (x: string) : option Term :=
  match g with
    | empty_subs => None
    | cons_subs g y a =>
      if string_eq x y then Some a else subs_lookup g x
  end.

Fixpoint without (g: Subs) (x: string) : Subs := (*delete a reference in substitution for variable value*)
  match g with
    | empty_subs => empty_subs
    | cons_subs g y a =>
      if string_eq x y
      then without g x
      else cons_subs (without g x) y a
  end.

Fixpoint subs1 (x: string) (s: Term) (t: Term) : Term :=
  match t with
    | var y      =>
      if string_eq x y then s else t
    | true       => true
    | false      => false
    | ifte a b c => ifte (subs1 x s a) (subs1 x s b) (subs1 x s c)
    | app f a    => app  (subs1 x s f) (subs1 x s a)
    | lamb y b   =>
      if string_eq x y then t else lamb y (subs1 x s b)
  end.

Fixpoint subs (g: Subs) (t: Term) : Term :=
  match t with
    | var x      =>
      match subs_lookup g x with
        | None   => var x
        | Some a => a
      end
    | true       => true
    | false      => false
    | ifte a b c => ifte (subs g a) (subs g b) (subs g c)
    | app f a    => app  (subs g f) (subs g a)
    | lamb x b   =>
      lamb x (subs (without g x) b)
  end.

Inductive Judgement : Set :=
  judgement : Ctx -> Term -> Tipe -> Judgement.

Notation "[ G |- a @ A ]" := (judgement G a A).

Inductive Deriv : Judgement -> Prop :=
  | d_true  : forall G: Ctx, Deriv [G |- true @ t_bool]
  | d_false : forall G: Ctx, Deriv [G |- false @ t_bool]
  | d_if    : forall G: Ctx,
              forall cond consq alt: Term,
              forall A: Tipe,
                   Deriv [G |- cond  @ t_bool]
                -> Deriv [G |- consq @ A]
                -> Deriv [G |- alt   @ A]
                -> Deriv [G |- ifte cond consq alt @ A]
  | d_var   : forall G: Ctx,
              forall x: string,
              forall A: Tipe,
                ctx_lookup G x = Some(A)
                -> Deriv [G |- (var x) @ A]
  | d_lamb  : forall G: Ctx,
              forall x: string,
              forall b: Term,
              forall A B: Tipe,
                Deriv [cons G x A |- b @ B]
                -> Deriv [G |- lamb x b @ A :-> B]
  | d_app   : forall G: Ctx,
              forall f a : Term,
              forall A B : Tipe,
                   Deriv [G |- f @ A :-> B]
                -> Deriv [G |- a @ A]
                -> Deriv [G |- app f a @ B].

Notation "# D" := (Deriv D) (at level 70).

Inductive Step : Term -> Term -> Prop :=
  | step_search_if : forall a b c a', Step a a' -> Step (ifte a b c) (ifte a' b c)
  | step_if_true : forall b c, Step (ifte true b c) b
  | step_if_false : forall b c, Step (ifte false b c) c
  | search1_app : forall f f' b, Step f f' -> Step (app f b) (app f' b)
  | search2_app : forall f b b', Step b b' -> value f -> Step (app f b) (app f b')
  | step_app : forall x a b, Step (app (lamb x b) a) (subs1 x a b).

Hint Constructors Step.

Inductive Halts : Term -> Prop :=
  | h_true : Halts true
  | h_false : Halts false
  | h_lambda : forall x b, Halts (lamb x b)
  | h_step : forall a b, Step a b -> Halts b -> Halts a.

Lemma subs1_lemma:
  forall x a A,
    #[empty |- a @ A]
    -> forall b B, #[cons empty x A |- b @ B]
                   -> #[empty |- subs1 x a b @ B].
Proof.
Admitted.

Theorem type_preservation:
  forall a a', Step a a' ->
               forall A, #[empty |- a @ A] -> #[empty |- a' @ A].
Proof.
  intros a a' step.
  induction step.
  - intros. inversion H. apply IHstep in H3.
    apply d_if; assumption.
  - intros. inversion H. assumption.
  - intros. inversion H. assumption.
  - intros. inversion H. subst. apply IHstep in H2.
    eapply d_app; eassumption.
  - intros. inversion H0; subst. apply IHstep in H6.
    eapply d_app; eassumption.
  - intros. inversion H; clear H; subst.
    inversion H2; subst.
    eapply subs1_lemma; eassumption.
Qed.

Theorem type_progress:
  forall a A,
    #[empty |- a @ A] -> value a \/ exists a', Step a a'.
Proof.
  intros. dependent induction H.
  - left. constructor.
  - left. constructor.
  - right.
    assert (value cond \/ exists a': Term, Step cond a').
    specialize IHDeriv1 with cond t_bool.
    apply IHDeriv1. reflexivity.
    destruct H2.
    { inversion H2; subst.
      { exists consq. apply step_if_true. }
      { exists alt.   apply step_if_false. }
      { inversion H. } }
    { elim H2; intro cond'; intros.
      exists (ifte cond' consq alt).
      constructor; assumption. }
  - left. constructor.
  - right.
    assert (value f \/ (exists a': Term, Step f a')).
    eapply IHDeriv1. reflexivity.
    destruct H1.
    { inversion H1; subst.
      { inversion H. }
      { inversion H. }
      { exists (subs1 x a0 b). constructor. } }
    { simpl. elim H1; intros f' step.
      exists (app f' a0). constructor. assumption. }
Qed.

Fixpoint SN (a: Term) (A: Tipe) : Prop :=
  match A with
    | t_bool  =>
      #[empty |- a @ t_bool]
      /\ Halts a
    | A :-> B =>
        #[empty |- a @ A :-> B]
        /\ Halts a
        /\ (forall b, SN b A -> SN (app a b) B)
  end.

Lemma sn__halt: forall a A, SN a A -> Halts a.
Proof.
  intros.
  destruct A; unfold SN in H.
  { destruct H; destruct a; assumption. }
  { simpl in H; fold SN in H.
    destruct H; destruct H0; assumption. }
Qed.
Hint Resolve sn__halt.

Lemma sn__type: forall a A, SN a A -> #[empty |- a @ A].
Proof.
  intros.
  destruct A; unfold SN in H; destruct H; assumption.
Qed.
Hint Resolve sn__type.

Definition subs_matches_ctx (g: Subs) (G: Ctx) : Prop :=
  forall x a A, subs_lookup g x = Some a /\
                ctx_lookup  G x = Some A /\
                SN a A.

Notation "g |= G" := (subs_matches_ctx g G) (at level 70).

Ltac destruct_sn :=
  repeat match goal with
    | [ H : SN _ t_bool |- _ ] =>
      simpl in H; destruct H
    | [ H : SN _ (_ :-> _) |- _ ] =>
      simpl in H; destruct H as (deriv & (halt & imp))
  end.

Ltac split_sn :=
  match goal with
    | [ |- SN _ t_bool ] =>
      simpl; split
    | [ |- SN _ (_ :-> _) ] =>
      simpl; split; try split
  end.

Lemma halts_if: forall a b c B,
                  Halts a -> Halts b -> Halts c ->
                  #[ empty |- a @ t_bool] -> 
                  #[ empty |- b @ B] -> 
                  #[ empty |- c @ B] ->
                  Halts (ifte a b c).
Proof.
  intros. induction H.
  - eapply h_step. apply step_if_true. assumption.
  - eapply h_step. apply step_if_false. assumption.
  - inversion H2.
  - apply h_step with (ifte b0 b c).
    apply step_search_if. assumption.
    apply IHHalts.
    eapply type_preservation; eassumption.
Qed.

Lemma weakening:
  forall a A G, #[empty |- a @ A] -> #[G |- a @ A].
Proof.
  Admitted.

Lemma if_halts:
  forall a b c,
    #[empty |- a @ t_bool] -> Halts a -> Halts b -> Halts c
    -> Halts (ifte a b c).
Proof.
  intros a b c H Ha Hb Hc.
  induction Ha.
  - eapply h_step. apply step_if_true. assumption.
  - eapply h_step. apply step_if_false. assumption.
  - inversion H.
  - 

Lemma sn_ind: forall g G a A,
                #[G |- a @ A] -> g |= G -> SN (subs g a) A.
Proof.
  intros.
  dependent induction H; destruct_sn; simpl.
  { split. apply d_true. apply h_true. }
  { split. apply d_false. apply h_false. }
  { intuition.
    destruct A; simpl.
    specialize IHDeriv1 with G cond t_bool.
    split; destruct_sn.
    { constructor.
      destruct IHDeriv1.
        reflexivity. assumption. assumption. assumption. assumption. }
    { destruct IHDeriv1.
        reflexivity. assumption.
      clear H3 H4 H2 H0 H1.
      induction H8; subst.
      { eapply h_step. apply step_if_true. assumption. }
      { eapply h_step. apply step_if_false. assumption. }
      { inversion H7. }
      { eapply type_preservation in H7.
        apply IHHalts in H7.
        apply h_step with (ifte b (subs g consq) (subs g alt)).
        apply step_search_if. assumption. assumption. assumption. } }
    { split.
        


    specialize IHDeriv1 with G cond t_bool.
    destruct IHDeriv1.
    { reflexivity. }
    { assumption. }
    { destruct A; simpl.
      { split.
        { constructor; auto. }
        { eapply halts_if; eauto. } }
      { split; try split.
        { constructor; auto. }
        { eapply halts_if; eauto. }
        { intros. subst.
    destruct IHDeriv1.
    assert (SN (subs g cond) 
  { intuition. destruct A; simpl; split.
    { apply sn__type in H3. apply sn__type in H4.
      apply d_if; try assumption.
      apply sn__type.
      eapply IHDeriv1. reflexivity. assumption. }
    { apply sn__halt in H3. apply sn__halt in H4.
      apply 

Lemma sn_if: forall a b c B,
               SN a t_bool -> SN b B -> SN c B -> SN (ifte a b c) B.
Proof.
  intros.
  destruct B; destruct_sn; split_sn.
  - constructor; auto.
  - eapply halts_if; eauto.
  - constructor; auto.
  - eapply halts_if; eauto.
  - intros.




Theorem strong_normalization: forall a A, #[empty |- a @ A] -> Halt a.
Proof.
  intros.