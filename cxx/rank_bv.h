#pragma once

#include "sux/bits/WordDynRankSel.hpp"
#include "sux/util/FenwickByteL.hpp"
#include "sux/util/Vector.hpp"

using namespace std;
using namespace sux;
using namespace sux::util;
using namespace sux::bits;

class RankBV {
private:
  mutable Vector<uint64_t> bitvect;
  mutable WordDynRankSel<FenwickByteL> bv;

public:
  RankBV(size_t size);
  ~RankBV();
  size_t size() const;
  bool get(size_t index) const;
  bool set(size_t index) const;
  bool clear(size_t index) const;
  bool toggle(size_t index) const;
  uint64_t rank(size_t index) const;
  size_t count_ones() const;
};

unique_ptr<RankBV> new_rank_bv(size_t size);
