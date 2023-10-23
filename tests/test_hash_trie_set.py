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
from imrc import HashSet
import pytest

HASH_MSG = "Not sure HashSet implements Hash, it has mutable methods"


def test_key_is_tuple():
    with pytest.raises(KeyError):
        HashSet().remove((1, 1))


def test_key_is_not_tuple():
    with pytest.raises(KeyError):
        HashSet().remove("asdf")


@pytest.mark.xfail(reason=HASH_MSG)
def test_supports_hash():
    assert hash(HashSet((1, 2))) == hash(HashSet(1, 2))


def test_empty_truthiness():
    assert HashSet([1])
    assert not HashSet()


def test_contains_elements_that_it_was_initialized_with():
    initial = [1, 2, 3]
    s = HashSet(initial)

    assert set(s) == set(initial)
    assert len(s) == len(set(initial))


def test_is_immutable():
    s1 = HashSet([1])
    s2 = s1.insert(2)

    assert s1 == HashSet([1])
    assert s2 == HashSet([1, 2])

    s3 = s2.remove(1)
    assert s2 == HashSet([1, 2])
    assert s3 == HashSet([2])


def test_remove_when_not_present():
    s1 = HashSet([1, 2, 3])
    with pytest.raises(KeyError):
        s1.remove(4)


def test_discard():
    s1 = HashSet((1, 2, 3))
    assert s1.discard(3) == HashSet((1, 2))
    assert s1.discard(4) == s1


def test_is_iterable():
    assert sum(HashSet([1, 2, 3])) == 6


def test_contains():
    s = HashSet([1, 2, 3])

    assert 2 in s
    assert 4 not in s


def test_supports_set_operations():
    s1 = HashSet([1, 2, 3])
    s2 = HashSet([3, 4, 5])

    assert s1 | s2 == HashSet([1, 2, 3, 4, 5])
    assert s1.union(s2) == s1 | s2

    assert s1 & s2 == HashSet([3])
    assert s1.intersection(s2) == s1 & s2

    assert s1 - s2 == HashSet([1, 2])
    assert s1.difference(s2) == s1 - s2

    assert s1 ^ s2 == HashSet([1, 2, 4, 5])
    assert s1.symmetric_difference(s2) == s1 ^ s2


def test_supports_set_comparisons():
    s1 = HashSet([1, 2, 3])
    s3 = HashSet([1, 2])
    s4 = HashSet([1, 2, 3])

    assert HashSet([1, 2, 3, 3, 5]) == HashSet([1, 2, 3, 5])
    assert s1 != s3

    assert s3 < s1
    assert s3 <= s1
    assert s3 <= s4

    assert s1 > s3
    assert s1 >= s3
    assert s4 >= s3


def test_repr():
    rep = repr(HashSet([1, 2]))
    assert rep == "HashSet({1, 2})" or rep == "HashSet({2, 1})"

    rep = repr(HashSet(["1", "2"]))
    assert rep == "HashSet({'1', '2'})" or rep == "HashSet({'2', '1'})"


def test_update():
    assert HashSet([1, 2, 3]).update([3, 4, 4, 5]) == HashSet([1, 2, 3, 4, 5])


def test_update_no_elements():
    s1 = HashSet([1, 2])
    assert s1.update([]) == s1


def test_iterable():
    assert HashSet(iter("a")) == HashSet(iter("a"))


def test_more_eq():
    # Non-pyrsistent-test-suite test
    o = object()

    assert HashSet([o]) == HashSet([o])
    assert HashSet([o, o]) == HashSet([o, o])
    assert HashSet([o]) == HashSet([o, o])
    assert HashSet() == HashSet([])
    assert not (HashSet([1, 2]) == HashSet([1, 3]))
    assert not (HashSet([o, 1]) == HashSet([o, o]))
    assert not (HashSet([]) == HashSet([o]))

    assert HashSet([1, 2]) != HashSet([1, 3])
    assert HashSet([]) != HashSet([o])
    assert not (HashSet([o]) != HashSet([o]))
    assert not (HashSet([o, o]) != HashSet([o, o]))
    assert not (HashSet([o]) != HashSet([o, o]))
    assert not (HashSet() != HashSet([]))


def test_more_set_comparisons():
    s = HashSet([1, 2, 3])

    assert s == s
    assert not (s < s)
    assert s <= s
    assert not (s > s)
    assert s >= s
