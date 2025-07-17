module agda.simple-test where

-- Simple identity function
id : {A : Set} → A → A
id x = x

-- Simple composition
_∘_ : {A B C : Set} → (B → C) → (A → B) → A → C
f ∘ g = λ x → f (g x)

-- Basic proof
id-compose : {A : Set} (x : A) → id (id x) ≡ x
  where
    _≡_ : {A : Set} → A → A → Set
    _≡_ = _≡_