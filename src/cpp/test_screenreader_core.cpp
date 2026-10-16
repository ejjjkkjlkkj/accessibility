#include "screenreader_core.hpp"
#include <cassert>
using namespace accessibility;
int main() {
    assert(announce({"bouton","Valider","",true,false})=="bouton: Valider");
    assert(announce({"bouton","Valider","",false,false}).empty());
    assert(announce({"champ","Volume","80 %",true,true})=="champ: Volume, 80 %, désactivé");
    EventQueue<2> q;
    assert(q.push({EventKind::Focus,"bouton","A","",true,false}));
    assert(q.push({EventKind::Focus,"bouton","B","",true,false}));
    assert(!q.push({EventKind::Focus,"bouton","C","",true,false}));
    Event e; assert(q.pop(e) && e.name=="A"); assert(q.pop(e) && e.name=="B"); assert(!q.pop(e));
}
