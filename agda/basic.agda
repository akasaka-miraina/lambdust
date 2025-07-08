module agda.basic where

-- Basic data type
data Bool : Set where
  true  : Bool
  false : Bool

-- Basic function
not : Bool → Bool
not true = false
not false = true

-- Equality type
data _≡_ {A : Set} (x : A) : A → Set where
  refl : x ≡ x

-- Simple proof
not-involution : (b : Bool) → not (not b) ≡ b
not-involution true = refl
not-involution false = refl