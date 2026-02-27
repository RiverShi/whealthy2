#import <UIKit/UIKit.h>
#import <WebKit/WebKit.h>
#import <objc/runtime.h>

// 在 WKWebView 初始化完成后，禁用 scrollView 的自动 inset 调整，
// 让 CSS viewport-fit=cover 能真正延伸到物理屏幕底部（home indicator 区域）
@implementation WKWebView (FullScreen)

+ (void)load {
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        Class class = [self class];
        SEL original = @selector(initWithFrame:configuration:);
        SEL swizzled = @selector(wealthy_initWithFrame:configuration:);
        Method originalMethod = class_getInstanceMethod(class, original);
        Method swizzledMethod = class_getInstanceMethod(class, swizzled);
        method_exchangeImplementations(originalMethod, swizzledMethod);
    });
}

- (instancetype)wealthy_initWithFrame:(CGRect)frame configuration:(WKWebViewConfiguration *)configuration {
    WKWebView *webView = [self wealthy_initWithFrame:frame configuration:configuration];
    if (webView) {
        webView.scrollView.contentInsetAdjustmentBehavior = UIScrollViewContentInsetAdjustmentNever;
    }
    return webView;
}

@end
