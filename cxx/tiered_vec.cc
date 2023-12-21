#include "tiered_vec.h"

#define implTieredVec(W, T)                                                    \
  TieredVec##W::TieredVec##W() {}                                              \
  TieredVec##W::~TieredVec##W() {}                                             \
  size_t TieredVec##W::len() const { return tiered.size; }                     \
  bool TieredVec##W::is_empty() const { return tiered.size == 0; }             \
  size_t TieredVec##W::capacity() const { return Layer##W::capacity; }         \
  T TieredVec##W::get(size_t idx) const { return tiered[idx]; }                \
  T TieredVec##W::update(size_t idx, T elem) const {                           \
    return helper<T, Layer##W>::replace(elem, (size_t)tiered.root, idx,        \
                                        tiered.info);                          \
  }                                                                            \
  void TieredVec##W::insert(size_t idx, T elem) const {                        \
    tiered.insert(idx, elem);                                                  \
  }                                                                            \
  void TieredVec##W::remove(size_t idx) const { tiered.remove(idx); }          \
  void TieredVec##W::insert_sorted(T elem) const {                             \
    tiered.insert_sorted(elem);                                                \
  }                                                                            \
  bool TieredVec##W::contains_sorted(T elem) const {                           \
    size_t left = 0, right = tiered.size;                                      \
    while (left < right) {                                                     \
      size_t mid = (left + right) / 2;                                         \
      T elem_mid = tiered[mid];                                                \
      if (elem < elem_mid) {                                                   \
        right = mid;                                                           \
      } else {                                                                 \
        if (elem == elem_mid) {                                                \
          return true;                                                         \
        }                                                                      \
        left = mid + 1;                                                        \
      }                                                                        \
    }                                                                          \
    return false;                                                              \
  }                                                                            \
  size_t TieredVec##W::index_sorted(T elem) const {                            \
    size_t left = 0, right = tiered.size;                                      \
    while (left < right) {                                                     \
      size_t mid = (left + right) / 2;                                         \
      T elem_mid = tiered[mid];                                                \
      if (elem < elem_mid) {                                                   \
        right = mid;                                                           \
      } else {                                                                 \
        if (elem == elem_mid) {                                                \
          return mid;                                                          \
        }                                                                      \
        left = mid + 1;                                                        \
      }                                                                        \
    }                                                                          \
    return tiered.size;                                                        \
  }                                                                            \
  unique_ptr<TieredVec##W> new_tiered_vec_##W() {                              \
    return unique_ptr<TieredVec##W>(new TieredVec##W());                       \
  }

implTieredVec(16, uint16_t);
implTieredVec(20, uint32_t);
implTieredVec(24, uint32_t);
implTieredVec(28, uint32_t);
implTieredVec(32, uint32_t);
