# Delta-Q utilities for Rust

the paper: <https://www.preprints.org/manuscript/202112.0132/v3>

The goal of this project is to provide tooling that is easy to use for designers of decentralised systems to model their network and assess communication timeliness and resource usage.
This should be supported by a web UI that allows modelling CDFs (cumulative distribution function), named outcome expressions, and constraints/expectations.
The underlying ΔQ expressions shall be used for export/import of the model, complemented with a library for CDF constructors (like step function etc.).

## Implementation plan

The first step is to provide a library for manipulating CDFs, offering all operations required by the theory; the internal representation will be discrete numerical, with later optimizations for constant parts at the beginning and end of the vector.

The second step yields an internal DSL for creating ΔQ expressions and printing them.

The third step provides evaluation of ΔQ expressions as defined in the paper.
This will later be expanded to include some exponentiation-like operator that simplifies expressing a randomly chosen repetition count for some sub-expression (as frequently occurs in gossip protocols).

The fourth step adds a web UI to expose the internal DSL to no-code users.
The interaction with a ΔQ expression shall closely resemble the refinement approach for system modelling as defined in the paper.
It will allow the system designer to see immediately the result of the current model and how its computed attenuation compares to the expectation or constraints.

In addition and in parallel to the above, the theory shall be better understood and where required enhanced to support not only timeliness analysis but also load prediction.
It is expected that while the same system model can be used for both aspects, the inputs for the load analysis need to be somewhat different from CDFs, i.e. they will likely require more information.

## Caveats

Since the timing CDFs don't model load dependence, they are only representative of the unloaded system.
The results of the load analysis will indicate where and under which conditions this assumption will be broken, but it isn't obvious how to feed that information back into a changed CDF to adapt the timeliness analysis to those circumstances.
