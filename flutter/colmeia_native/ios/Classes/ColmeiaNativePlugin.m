#import "ColmeiaNativePlugin.h"
#if __has_include(<colmeia_native/colmeia_native-Swift.h>)
#import <colmeia_native/colmeia_native-Swift.h>
#else
// Support project import fallback if the generated compatibility header
// is not copied when this plugin is created as a library.
// https://forums.swift.org/t/swift-static-libraries-dont-copy-generated-objective-c-header/19816
#import "colmeia_native-Swift.h"
#endif

@implementation ColmeiaNativePlugin
+ (void)registerWithRegistrar:(NSObject<FlutterPluginRegistrar>*)registrar {
  [SwiftColmeiaNativePlugin registerWithRegistrar:registrar];
}
@end
