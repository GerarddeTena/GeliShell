/**
 * @name Forbidden unsafe block
 * @description Detects all unsafe blocks. Project policy (Cargo.toml: -D unsafe_code) forbids
 *              unsafe code everywhere. Any unsafe block is a policy violation.
 * @kind problem
 * @problem.severity error
 * @id rust/forbidden-unsafe-block
 * @tags security
 *       correctness
 */

import rust

from BlockExpr b
where b.isUnsafe()
select b, "Unsafe block detected — project policy forbids unsafe code (Cargo.toml: -D unsafe_code)"