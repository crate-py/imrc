"""
Modified from the pyrsistent test suite.

Pre-modification, these were MIT licensed, and are copyright:

    Copyright (c) 2022 Tobias Gustafsson

    Permission is hereby granted, free of charge, to any person
    obtaining a copy of this software and associated documentation
    files (the "Software"), to deal in the Software without
    restriction, including without limitation the rights to use,
    copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following
    conditions:

    The above copyright notice and this permission notice shall be
    included in all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
    EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
    OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
    NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
    HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
    WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
    OTHER DEALINGS IN THE SOFTWARE.
"""
import pytest

from imrc import Vector

HASH_MSG = "Not sure Vector implements Hash, it has mutable methods"


def test_literalish_works():
    assert Vector(1, 2, 3) == Vector([1, 2, 3])


def test_first_and_rest():
    pl = Vector([1, 2])
    assert pl.first == 1
    assert pl.rest.first == 2
    assert pl.rest.rest == Vector()


def test_instantiate_large_list():
    assert Vector(range(1000)).first == 0


def test_iteration():
    assert list(Vector()) == []
    assert list(Vector([1, 2, 3])) == [1, 2, 3]


def test_push_front():
    assert Vector([1, 2, 3]).push_front(0) == Vector([0, 1, 2, 3])


def test_push_front_empty_list():
    assert Vector().push_front(0) == Vector([0])


def test_truthiness():
    assert Vector([1])
    assert not Vector()


def test_len():
    assert len(Vector([1, 2, 3])) == 3
    assert len(Vector()) == 0


def test_first_illegal_on_empty_list():
    with pytest.raises(IndexError):
        Vector().first


def test_rest_return_self_on_empty_list():
    assert Vector().rest == Vector()


def test_reverse():
    assert reversed(Vector([1, 2, 3])) == Vector([3, 2, 1])

    assert reversed(Vector()) == Vector()


def test_inequality():
    assert Vector([1, 2]) != Vector([1, 3])
    assert Vector([1, 2]) != Vector([1, 2, 3])
    assert Vector() != Vector([1, 2, 3])


def test_repr():
    assert str(Vector()) == "Vector([])"
    assert str(Vector([1, 2, 3])) in "Vector([1, 2, 3])"


@pytest.mark.xfail(reason=HASH_MSG)
def test_hashing():
    assert hash(Vector([1, 2])) == hash(Vector([1, 2]))
    assert hash(Vector([1, 2])) != hash(Vector([2, 1]))


def test_sequence():
    m = Vector("asdf")
    assert m == Vector(["a", "s", "d", "f"])
