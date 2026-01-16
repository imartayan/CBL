#pragma once

#define ARRAY 1
#define LEVEL 1
#define PACK 1

#include "tiered-vector/include/templated_tiered.h"
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
// 5 + 5 + 5 + 6 + 9 = 30
using Layer30 =
    LayerItr<LayerEnd, Layer<32, Layer<32, Layer<32, Layer<64, Layer<512>>>>>>;
// 5 + 6 + 6 + 6 + 9 = 32
using Layer32 =
    LayerItr<LayerEnd, Layer<32, Layer<64, Layer<64, Layer<64, Layer<512>>>>>>;

#define defTieredVec(W, T)                                                     \
  class TieredVec##W {                                                         \
  public:                                                                      \
    TieredVec##W() {}                                                          \
    ~TieredVec##W() {}                                                         \
    size_t len() const { return tiered.size; }                                 \
    bool is_empty() const { return tiered.size == 0; }                         \
    size_t capacity() const { return Layer##W::capacity; }                     \
    T get(size_t idx) const { return tiered[idx]; }                            \
    T update(size_t idx, T elem) const {                                       \
      return helper<T, Layer##W>::replace(elem, (size_t)tiered.root, idx,      \
                                          tiered.info);                        \
    }                                                                          \
    void insert(size_t idx, T elem) const { tiered.insert(idx, elem); }        \
    void remove(size_t idx) const { tiered.remove(idx); }                      \
    void insert_sorted(T elem) const { tiered.insert_sorted(elem); }           \
    bool contains_sorted(T elem) const {                                       \
      size_t left = 0, right = tiered.size;                                    \
      while (left < right) {                                                   \
        size_t mid = (left + right) / 2;                                       \
        T elem_mid = tiered[mid];                                              \
        if (elem < elem_mid) {                                                 \
          right = mid;                                                         \
        } else {                                                               \
          if (elem == elem_mid) {                                              \
            return true;                                                       \
          }                                                                    \
          left = mid + 1;                                                      \
        }                                                                      \
      }                                                                        \
      return false;                                                            \
    }                                                                          \
    size_t index_sorted(T elem) const {                                        \
      size_t left = 0, right = tiered.size;                                    \
      while (left < right) {                                                   \
        size_t mid = (left + right) / 2;                                       \
        T elem_mid = tiered[mid];                                              \
        if (elem < elem_mid) {                                                 \
          right = mid;                                                         \
        } else {                                                               \
          if (elem == elem_mid) {                                              \
            return mid;                                                        \
          }                                                                    \
          left = mid + 1;                                                      \
        }                                                                      \
      }                                                                        \
      return tiered.size;                                                      \
    }                                                                          \
                                                                               \
  private:                                                                     \
    mutable Seq::Tiered<T, Layer##W> tiered;                                   \
  };

defTieredVec(16, uint16_t);
defTieredVec(20, uint32_t);
defTieredVec(24, uint32_t);
defTieredVec(28, uint32_t);
defTieredVec(30, uint32_t);
defTieredVec(32, uint32_t);
