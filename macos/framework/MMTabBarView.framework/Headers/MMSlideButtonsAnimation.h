//
//  MMSlideButtonsAnimation.h
//  MMTabBarView
//
//  Created by Michael Monscheuer on 9/12/12.
//
//

#import <Cocoa/Cocoa.h>

@interface MMSlideButtonsAnimation : NSViewAnimation

- (instancetype)initWithTabBarButtons:(NSSet *)buttons NS_DESIGNATED_INITIALIZER;

- (void)addAnimationDictionary:(NSDictionary *)aDict;

@end
