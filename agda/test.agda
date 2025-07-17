module agda.test where

open import Data.Nat
open import Relation.Binary.PropositionalEquality

-- Simple proof that addition is commutative for 0
proof : ∀ (n : ℕ) → n + 0 ≡ 0 + n
proof zero = refl
proof (suc n) = cong suc (proof n)