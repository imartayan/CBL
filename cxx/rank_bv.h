#pragma once

#include "sux/sux/bits/WordDynRankSel.hpp"
#include "sux/sux/util/FenwickByteL.hpp"
#include "sux/sux/util/Vector.hpp"

using namespace std;
using namespace sux;
using namespace sux::util;
using namespace sux::bits;

using BV = Vector<uint64_t>;

// BV merge_bv(BV fst, BV snd);
// BV intersect_bv(BV fst, BV snd);

class RankBV {
private:
  mutable BV bitvect;
  mutable WordDynRankSel<FenwickByteL> bv;

public:
  RankBV(size_t size) : bitvect((size + 63) / 64), bv(&bitvect, size) {}
  // RankBV(size_t size, BV &bitvect);
  ~RankBV() {}

  // Delete copy operators
  RankBV(const RankBV &) = delete;
  RankBV &operator=(const RankBV &) = delete;

  size_t size() const { return bv.size(); }
  bool get(size_t index) const {
    return bv.bitvector()[index / 64] & (1ULL << (index % 64));
  }
  bool set(size_t index) const { return bv.set(index); }
  bool clear(size_t index) const { return bv.clear(index); }
  bool toggle(size_t index) const { return bv.toggle(index); }
  uint64_t rank(size_t index) const { return bv.rank(index); }
  size_t count_ones() const { return bv.rank(bv.size() - 1); }
  // RankBV merge(RankBV &other);
  // RankBV intersect(RankBV &other);
};
