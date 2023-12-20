#include "rank_bv.h"

RankBV::RankBV(size_t size) : bitvect((size + 63) / 64), bv(&bitvect, size) {}

RankBV::~RankBV() {}

size_t RankBV::size() const { return bv.size(); }

bool RankBV::get(size_t index) const {
  return bv.bitvector()[index / 64] & (1ULL << (index % 64));
}

bool RankBV::set(size_t index) const { return bv.set(index); }

bool RankBV::clear(size_t index) const { return bv.clear(index); }

bool RankBV::toggle(size_t index) const { return bv.toggle(index); }

uint64_t RankBV::rank(size_t index) const { return bv.rank(index); }

size_t RankBV::count_ones() const { return bv.rank(bv.size() - 1); }

unique_ptr<RankBV> new_rank_bv(size_t size) {
  return unique_ptr<RankBV>(new RankBV(size));
}
