#import "ScreenReaderCore.h"
@implementation ADMAccessibleNode
- (instancetype)init {
    self = [super init];
    if (self) { _role = @""; _name = @""; _value = @""; _focused = NO; _disabled = NO; }
    return self;
}
@end
NSString *ADMAnnounceNode(ADMAccessibleNode *node) {
    if (!node.focused) return @"";
    NSString *role = node.role.length ? node.role : @"element";
    NSMutableString *text = [NSMutableString stringWithString:role];
    if (node.name.length) [text appendFormat:@": %@", node.name];
    if (node.value.length) [text appendFormat:@", %@", node.value];
    if (node.disabled) [text appendString:@", disabled"];
    return [text copy];
}
