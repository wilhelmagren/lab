import sys
from typing import Tuple


class SuffixArray:
    """ """

    def __init__(self, s: str) -> None:
        self._s = s
        self._n = len(s)

        suffixes = []
        array = []
        for suffix, idx in sorted((s[i:], i) for i in range(len(s))):
            suffixes.append(suffix)
            array.append(idx)

        self._suffixes = suffixes
        self._array = array

    def search(self, q: str) -> Tuple[int, int]:
        """ """

        left = 0
        right = self._n
        while left < right:
            mid = (left + right) // 2
            if q > self._suffixes[self._array[mid]]:
                left = mid + 1
            else:
                right = mid

        st_idx = left

        right = self._n
        while left < right:
            mid = (left + right) // 2
            if self._suffixes[self._array[mid]].startswith(q):
                left = mid + 1
            else:
                right = mid

        return (st_idx, right)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("You need to provide a string S and query q")
        exit(1)

    sq = SuffixArray(sys.argv[1])
    q = sys.argv[2]

    print(sq.search(q))

