
arrayvec
========

A vector with fixed capacity.  Requires Rust 1.2+.

Please read the `API documentation here`__

__ http://bluss.github.io/arrayvec

|build_status|_ |crates|_ |crates2|_

.. |build_status| image:: https://travis-ci.org/bluss/arrayvec.svg
.. _build_status: https://travis-ci.org/bluss/arrayvec

.. |crates| image:: http://meritbadge.herokuapp.com/arrayvec
.. _crates: https://crates.io/crates/arrayvec

.. |crates2| image:: http://meritbadge.herokuapp.com/nodrop
.. _crates2: https://crates.io/crates/nodrop

Recent Changes (arrayvec)
-------------------------

- 0.3.16

  - Added method ``.retain()`` to ``ArrayVec``.
  - Added methods ``.as_slice(), .as_mut_slice()`` to ``ArrayVec`` and ``.as_str()``
    to ``ArrayString``.

- 0.3.15

  - Add feature std, which you can opt out of to use ``no_std`` (requires Rust 1.6
    to opt out).
  - Implement ``Clone::clone_from`` for ArrayVec and ArrayString

- 0.3.14

  - Add ``ArrayString::from(&str)``

- 0.3.13

  - Added ``DerefMut`` impl for ``ArrayString``.
  - Added method ``.simplify()`` to drop the element for ``CapacityError``.
  - Added method ``.dispose()`` to ``ArrayVec``

- 0.3.12

  - Added ArrayString, a fixed capacity analogy of String

- 0.3.11

  - Added trait impls Default, PartialOrd, Ord, Write for ArrayVec

- 0.3.10

  - Go back to using external NoDrop, fixing a panic safety bug (issue #3)

- 0.3.8

  - Inline the non-dropping logic to remove one drop flag in the
    ArrayVec representation.

- 0.3.7

  - Added method .into_inner()
  - Added unsafe method .set_len()

Recent Changes (nodrop)
-----------------------

- 0.1.6

  - Add feature std, which you can opt out of to use ``no_std``.

- 0.1.5

  - Added crate feature ``use_needs_drop`` which is a nightly-only
    optimization, which skips overwriting if the inner value does not need
    drop.


License
=======

Dual-licensed to be compatible with the Rust project.

Licensed under the Apache License, Version 2.0
http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
http://opensource.org/licenses/MIT, at your
option. This file may not be copied, modified, or distributed
except according to those terms.


