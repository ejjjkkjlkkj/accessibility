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
    NSMutableArray<NSString *> *parts = [NSMutableArray arrayWithObject:role];
    if (node.name.length) [parts addObject:node.name];
    if (node.value.length) [parts addObject:node.value];
    NSString *text = parts.count > 1
      ? [NSString stringWithFormat:@"%@: %@", parts[0], parts[1]]
      : parts[0];
    if (parts.count > 2) text = [text stringByAppendingFormat:@", %@", parts[2]];
    if (node.disabled) text = [text stringByAppendingString:@", disabled"];
    return text;
}
