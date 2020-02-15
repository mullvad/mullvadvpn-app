# Introduction

`winfw` implements a number of policies for configuring the Windows Filtering Platform (WFP).

# Organization of sublayers

`winfw` uses a design that involves two different types of sublayers:

- The baseline sublayer
- Other sublayers

## Baseline sublayer

The baseline sublayer is weighted the highest to ensure it sees all traffic first. It contains a large number of permit-filters, with a different subset of them being activated by different policies. The permit-filters are all weighted the same and have the highest possible weight. It doesn't matter which filter sees the traffic first. If traffic is matched by a permit-filter, it's "lifted" out of the sublayer and processing is resumed with the next sublayer.

The baseline sublayer also contains a set of blocking filters that match all traffic. These filters are weighted the lowest within the sublayer. A blocking verdict is final and any traffic matched will be dropped.

The idea is that the primary sublayer (baseline sublayer) shapes the traffic to be more predictable for filters in subsequent sublayers.

## Other sublayers

Beyond the baseline sublayer, there's also the "other" type of sublayer. These sublayers are all weighted the same and slightly lower than the baseline sublayer. These sublayers focus on a specific type of traffic.

Same as the baseline sublayer, these sublayers use a design with highly weighted permit-filters and lower weighted blocking filters.

Since these sublayers deal only with a very specific type of traffic it's simple to express brief and exact filters. As an example, we define a sublayer for non-tunnel DNS traffic that has permit-filters for exactly the kind of DNS traffic we want to permit. This is complemented with blocking filters that match all non-tunnel DNS traffic.

## Advantages of current design

- Predictable sublayer weights.
- Predictable filter weights.
- Short and exact filter condition definitions.
- Removes the need to express "and also not equal to", something which is not possible in WFP.
