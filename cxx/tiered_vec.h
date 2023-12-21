#pragma once

#define ARRAY 1
#define LEVEL 1
#define PACK 1

#include "templated_tiered.h"
#include <memory>
#include <stdlib.h>

using namespace std;
using namespace Seq;

// 3 + 3 + 3 + 7 = 16
using Layer16 = LayerItr<LayerEnd, Layer<8, Layer<8, Layer<8, Layer<128>>>>>;
// 4 + 4 + 4 + 8 = 20
using Layer20 = LayerItr<LayerEnd, Layer<16, Layer<16, Layer<16, Layer<256>>>>>;
// 4 + 4 + 4 + 4 + 8 = 24
using Layer24 =
    LayerItr<LayerEnd, Layer<16, Layer<16, Layer<16, Layer<16, Layer<256>>>>>>;
// 5 + 5 + 5 + 5 + 8 = 28
using Layer28 =
    LayerItr<LayerEnd, Layer<32, Layer<32, Layer<32, Layer<32, Layer<256>>>>>>;
// 5 + 6 + 6 + 6 + 9 = 32
using Layer32 =
    LayerItr<LayerEnd, Layer<32, Layer<64, Layer<64, Layer<64, Layer<512>>>>>>;

#define declTieredVec(W, T)                                                    \
  class TieredVec##W {                                                         \
  public:                                                                      \
    TieredVec##W();                                                            \
    ~TieredVec##W();                                                           \
    size_t len() const;                                                        \
    bool is_empty() const;                                                     \
    size_t capacity() const;                                                   \
    T get(size_t idx) const;                                                   \
    T update(size_t idx, T elem) const;                                        \
    void insert(size_t idx, T elem) const;                                     \
    void remove(size_t idx) const;                                             \
    void insert_sorted(T elem) const;                                          \
    bool contains_sorted(T elem) const;                                        \
    size_t index_sorted(T elem) const;                                         \
                                                                               \
  private:                                                                     \
    mutable Seq::Tiered<T, Layer##W> tiered;                                   \
  };                                                                           \
  unique_ptr<TieredVec##W> new_tiered_vec_##W();

declTieredVec(16, uint16_t);
declTieredVec(20, uint32_t);
declTieredVec(24, uint32_t);
declTieredVec(28, uint32_t);
declTieredVec(32, uint32_t);
