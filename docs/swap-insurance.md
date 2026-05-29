# Swap Insurance

## Overview
Opt-in insurance policies for swap transactions with tiered coverage and automated claim assessment.

## Policy Types

| Type     | Multiplier | Covered Events                                   |
|----------|------------|--------------------------------------------------|
| BASIC    | 1.0×       | Non-delivery                                     |
| STANDARD | 1.6×       | Non-delivery, Item-not-as-described              |
| PREMIUM  | 2.5×       | Non-delivery, INAD, Fraud, Force majeure         |

Premium = `swapValue × 1% × multiplier × riskFactor`

## Risk Factors
| Condition              | Adjustment |
|------------------------|------------|
| Seller rep < 300       | +1.2×      |
| Seller rep 300–499     | +0.6×      |
| Seller rep ≥ 700       | −0.2×      |
| Swap value > $50k      | +0.5×      |
| Swap value > $200k     | +0.5×      |
| Seller swap count < 5  | +0.4×      |

## Claims
- 5% deductible applied to approved claims
- Evidence string required for approval
- Payout capped at `coverageAmount` (= `swapValue`)

## Usage
```js
const { calculatePremium, issuePolicy, fileClaim, POLICY_TYPES, COVERAGE_EVENTS } =
  require('./src/insurance/swapInsurance');

const { premium } = calculatePremium(10_000, POLICY_TYPES.STANDARD, { sellerReputationScore: 620 });
const policy = issuePolicy({ swapId: 'swap-1', policyType: POLICY_TYPES.STANDARD, swapValue: 10_000, buyerId: 'b1' });
const claim  = fileClaim(policy, { event: COVERAGE_EVENTS.NON_DELIVERY, claimedAmount: 8000, evidence: 'proof.pdf' });
```
