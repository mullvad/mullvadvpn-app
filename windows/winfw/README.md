# Introduction

`winfw` implements a number of policies for configuring the Windows Filtering Platform (WFP).

# Organization of sublayers

In its initialized state, `winfw` uses a design that involves two different types of sublayers:

- The baseline sublayer
- Other sublayers

When `winfw` is deinitialized, it may create a persistent sublayer to continue applying some policies. Other sublayers and their filters are removed at this time.

## Baseline sublayer

The baseline sublayer is weighted the highest to ensure it sees all traffic first. It contains a large number of permit-filters, with a different subset of them being activated by different policies. The permit-filters are all weighted the same and have the highest possible weight. It doesn't matter which filter sees the traffic first. If traffic is matched by a permit-filter, it's "lifted" out of the sublayer and processing is resumed with the next sublayer.

The baseline sublayer also contains a set of blocking filters that match all traffic. These filters are weighted the lowest within the sublayer. A blocking verdict is final and any traffic matched will be dropped.

The idea is that the primary sublayer (baseline sublayer) shapes the traffic to be more predictable for filters in subsequent sublayers.

## Other sublayers

Beyond the baseline sublayer, there's also the "other" type of sublayer. These sublayers are all weighted the same and slightly lower than the baseline sublayer. These sublayers focus on a specific type of traffic.

Same as the baseline sublayer, these sublayers use a design with highly weighted permit-filters and lower weighted blocking filters.

As an example, we have a sublayer that's dedicated to filtering DNS. Traffic that's not related to DNS will still be sent through it, but all the filters we install must deal only with DNS. This way we can install permit-filters with specific conditions that effectively whitelist the traffic we deem safe. To round it off there's a lower-weighted blocking filter that blocks all DNS.

## Persistent sublayer

The persistent sublayer is only active when `winfw` is deinitialized and instructed to continue enforcing a blocking policy. It has the highest weight possible and only contains blocking filters that match all traffic. These filters ensure that all traffic will be blocked until `winfw` is reinitialized, including at boot-time before BFE is running. Unlike the other sublayers, this sublayer persists even if BFE is restarted (e.g., by rebooting).

## Advantages of current design

- Predictable sublayer weights.
- Predictable filter weights.
- Short and exact filter condition definitions.
- Removes the need to express logical "and" for same-type conditions, something which is not possible in WFP.
