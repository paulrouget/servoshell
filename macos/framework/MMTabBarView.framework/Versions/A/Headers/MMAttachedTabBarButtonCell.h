//
//  MMAttachedTabBarButtonCell.h
//  MMTabBarView
//
//  Created by Michael Monscheuer on 9/5/12.
//
//

#import "MMTabBarButtonCell.h"

#import "MMAttachedTabBarButton.h"

@interface MMAttachedTabBarButtonCell : MMTabBarButtonCell

@property (assign) BOOL isOverflowButton;

@property (assign) MMAttachedTabBarButton *controlView;

@end
