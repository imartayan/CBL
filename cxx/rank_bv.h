#pragma once

#include "sux/sux/bits/WordDynRankSel.hpp"
#include "sux/sux/util/FenwickByteL.hpp"
#include "sux/sux/util/Vector.hpp"

using namespace std;
using namespace sux;
using namespace sux::util;
using namespace sux::bits;

using BV = Vector<uint64_t>;

class RankBV {
private:
  mutable BV bitvector;
  mutable WordDynRankSel<FenwickByteL> rbv;

public:
  RankBV(size_t size) : bitvector((size + 63) / 64), rbv(&bitvector, size) {}

  // Delete copy operators
  RankBV(const RankBV &) = delete;
  RankBV &operator=(const RankBV &) = delete;

  size_t size() const { return rbv.size(); }
  bool get(size_t index) const {
    return rbv.bitvector()[index / 64] & (1ULL << (index % 64));
  }
  bool set(size_t index) const { return rbv.set(index); }
  bool clear(size_t index) const { return rbv.clear(index); }
  bool toggle(size_t index) const { return rbv.toggle(index); }
  uint64_t rank(size_t index) const { return rbv.rank(index); }
  size_t count_ones() const { return rbv.rank(rbv.size() - 1); }

  void merge(RankBV &other) const {
    assert(rbv.size() == other.rbv.size());
    for (size_t i = 0; i < bitvector.size(); i++) {
      rbv.update(i, bitvector[i] | other.bitvector[i]);
    }
  }

  void intersect(RankBV &other) const {
    assert(rbv.size() == other.rbv.size());
    for (size_t i = 0; i < bitvector.size(); i++) {
      rbv.update(i, bitvector[i] & other.bitvector[i]);
    }
  }

  void difference(RankBV &other) const {
    assert(rbv.size() == other.rbv.size());
    for (size_t i = 0; i < bitvector.size(); i++) {
      rbv.update(i, bitvector[i] & ~other.bitvector[i]);
    }
  }

  void symmetric_difference(RankBV &other) const {
    assert(rbv.size() == other.rbv.size());
    for (size_t i = 0; i < bitvector.size(); i++) {
      rbv.update(i, bitvector[i] ^ other.bitvector[i]);
    }
  }
};
