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
  size_t num_blocks() const { return bitvector.size(); }
  uint64_t get_block(size_t block_index) const {
    return bitvector[block_index];
  }
  void update_block(size_t block_index, uint64_t value) const {
    rbv.update(block_index, value);
  }
};
