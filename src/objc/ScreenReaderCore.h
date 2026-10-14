#import <Foundation/Foundation.h>
NS_ASSUME_NONNULL_BEGIN
@interface ADMAccessibleNode : NSObject
@property(nonatomic, copy) NSString *role;
@property(nonatomic, copy) NSString *name;
@property(nonatomic, copy) NSString *value;
@property(nonatomic) BOOL focused;
@property(nonatomic) BOOL disabled;
@end
FOUNDATION_EXPORT NSString *ADMAnnounceNode(ADMAccessibleNode *node);
NS_ASSUME_NONNULL_END
