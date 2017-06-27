//
//  MMTabBarButton.h
//  MMTabBarView
//
//  Created by Michael Monscheuer on 9/5/12.
//
//

#import <Cocoa/Cocoa.h>

#import "MMTabBarView.h"
#import "MMRolloverButton.h"
#import "MMProgressIndicator.h"
#import "MMTabBarButton.Common.h"

@class MMTabBarView;
@class MMTabBarButtonCell;

@protocol MMTabStyle;

@interface MMTabBarButton : MMRolloverButton

- (instancetype)initWithFrame:(NSRect)frame;

#pragma mark Properties

@property (assign) NSRect stackingFrame;
@property (strong) MMRolloverButton *closeButton;
@property (assign) SEL closeButtonAction;
@property (readonly, strong) MMProgressIndicator *indicator;

- (MMTabBarButtonCell *)cell;
- (void)setCell:(MMTabBarButtonCell *)aCell;

- (MMTabBarView *)tabBarView;

#pragma mark Update Cell

- (void)updateCell;
- (void)updateImages;

#pragma mark Dividers

@property (readonly) BOOL shouldDisplayLeftDivider;
@property (readonly) BOOL shouldDisplayRightDivider;

#pragma mark Determine Sizes

- (CGFloat)minimumWidth;
- (CGFloat)desiredWidth;

#pragma mark Interfacing Cell

@property (strong) id <MMTabStyle> style;
@property (assign) MMTabStateMask tabState;

@property (strong) NSImage *icon;
@property (strong) NSImage *largeImage;

@property (assign) BOOL showObjectCount;
@property (assign) NSInteger objectCount;

@property (strong) NSColor *objectCountColor;

@property (assign) BOOL isEdited;
@property (assign) BOOL isProcessing;

#pragma mark Close Button Support

@property (readonly) BOOL shouldDisplayCloseButton;
@property (assign) BOOL hasCloseButton;
@property (assign) BOOL suppressCloseButton;

@end
