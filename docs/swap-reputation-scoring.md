# Swap Reputation Scoring

## Overview
`calculateReputationScore(input)` produces a 0–1000 reputation score for any
swap participant based on their historical swap behaviour.

## Score Factors

| Factor            | Range        | Description                                    |
|-------------------|--------------|------------------------------------------------|
| Completion rate   | 0 – 200 pts  | Completed / initiated swaps                    |
| Dispute penalty   | -150 – 0 pts | Dispute rate on completed swaps                |
| Weighted rating   | 0 – 300 pts  | 1–5 star avg, recency-weighted (90d half-life) |
| Tenure bonus      | 0 – 100 pts  | Log-scale up to 2 years                        |
| Volume bonus      | 0 – 100 pts  | √n scale up to 200 swaps                      |
| Cancellation pen. | -150 – 0 pts | Recency-weighted cancellation count            |

## Tiers
| Score    | Tier     |
|----------|----------|
| 850–1000 | Platinum |
| 700–849  | Gold     |
| 550–699  | Silver   |
| 400–549  | Bronze   |
| < 400    | New      |

## Usage
```js
const { calculateReputationScore } = require('./src/reputation/swapReputationScorer');

const result = calculateReputationScore({
  participantId: 'user-42',
  accountCreatedAt: '2022-01-01',
  history: [ /* HistoryEntry[] */ ],
});
console.log(result.score, result.tier);
```
