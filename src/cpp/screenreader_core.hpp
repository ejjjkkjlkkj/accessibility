#pragma once
#include <array>
#include <cstddef>
#include <string>
#include <string_view>

namespace accessibility {
struct Node { std::string_view role, name, value; bool focused=false, disabled=false; };
inline std::string announce(const Node& n) {
    if (!n.focused) return {};
    std::string out(n.role.empty() ? "élément" : n.role);
    if (!n.name.empty()) { out += ": "; out += n.name; }
    if (!n.value.empty()) { out += ", "; out += n.value; }
    if (n.disabled) out += ", désactivé";
    return out;
}
enum class EventKind { Focus, ValueChanged, StateChanged };
struct Event { EventKind kind{}; std::string role, name, value; bool focused=false, disabled=false; };
template<std::size_t Capacity>
class EventQueue {
    static_assert(Capacity > 0);
    std::array<Event, Capacity> items_{};
    std::size_t head_=0, size_=0;
public:
    bool push(const Event& e) {
        if (size_ == Capacity) return false;
        items_[(head_+size_)%Capacity]=e; ++size_; return true;
    }
    bool pop(Event& out) {
        if (!size_) return false;
        out=items_[head_]; head_=(head_+1)%Capacity; --size_; return true;
    }
    std::size_t size() const noexcept { return size_; }
    bool empty() const noexcept { return size_==0; }
};
}
