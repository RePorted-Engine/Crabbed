// The iPhone build of Horizon's simulation core, showing what the crate answers on this device.
//
// This is not the game: there is no window, no GPU and no resource pack here, and the client crate
// does not compile for iOS yet (its clipboard dependency drags in the Wayland stack). What this app
// does is call horizon_probe() once, from the app's own process, and print the report of four of
// Horizon's subsystems running on arm64 Apple silicon.
#import <UIKit/UIKit.h>

#include "horizon_ios.h"

@interface HorizonReportViewController : UIViewController
@end

@implementation HorizonReportViewController {
  UITextView *_report;
}

- (void)viewDidLoad {
  [super viewDidLoad];
  self.title = @"On this iPhone";
  self.view.backgroundColor = [UIColor blackColor];

  _report = [[UITextView alloc] initWithFrame:self.view.bounds];
  _report.autoresizingMask = UIViewAutoresizingFlexibleWidth | UIViewAutoresizingFlexibleHeight;
  _report.editable = NO;
  _report.font = [UIFont monospacedSystemFontOfSize:13.0 weight:UIFontWeightRegular];
  _report.textColor = [UIColor whiteColor];
  _report.backgroundColor = [UIColor blackColor];
  _report.text = @"Horizon's simulation core does not answer.";
  [self.view addSubview:_report];

  char buffer[HORIZON_PROBE_CAPACITY];
  ptrdiff_t written = horizon_probe(buffer, sizeof buffer);
  if (written < 0) {
    _report.text = [NSString stringWithFormat:
        @"horizon_probe() refused the buffer: %td (needs %zu bytes)",
        written, horizon_probe_capacity()];
    return;
  }
  NSString *text = [NSString stringWithUTF8String:buffer];
  if (text == nil) {
    _report.text = @"horizon_probe() answered with something that is not UTF-8.";
    return;
  }
  _report.text = [NSString stringWithFormat:@"%@\n\nReport read from %ld bytes of Rust, "
                                            @"linked into this app as libhorizon_ios.a.",
                  text, (long)written];
}

@end

@interface HorizonAppDelegate : NSObject <UIApplicationDelegate>
@property(nonatomic, strong) UIWindow *window;
@end

@implementation HorizonAppDelegate

- (BOOL)application:(UIApplication *)application
    didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
  HorizonReportViewController *report = [[HorizonReportViewController alloc] init];
  UINavigationController *navigation =
      [[UINavigationController alloc] initWithRootViewController:report];
  self.window = [[UIWindow alloc] initWithFrame:[UIScreen mainScreen].bounds];
  self.window.rootViewController = navigation;
  [self.window makeKeyAndVisible];
  return YES;
}

@end

int main(int argc, char *argv[]) {
  @autoreleasepool {
    return UIApplicationMain(argc, argv, nil, @"HorizonAppDelegate");
  }
}
