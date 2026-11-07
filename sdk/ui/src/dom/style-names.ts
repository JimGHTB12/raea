// The CSS properties a style declaration has in a browser, by their camelCase names: `name in el.style` is true for
// these, `--` custom properties and webkit-prefixed names, and false for anything else (textContent, value, ...), so a
// library that feature-tests a style property (GSAP's CSSPlugin) sends the rest to the element. Taken from Chrome 148's
// CSSStyleDeclaration (its webkit-prefixed names matched by their prefix instead).
const NAMES = new Set(
  (
    "accentColor additiveSymbols alignContent alignItems alignmentBaseline alignSelf all anchorName anchorScope " +
    "animation animationComposition animationDelay animationDirection animationDuration animationFillMode " +
    "animationIterationCount animationName animationPlayState animationRange animationRangeEnd animationRangeStart " +
    "animationTimeline animationTimingFunction animationTrigger appearance appRegion ascentOverride aspectRatio " +
    "backdropFilter backfaceVisibility background backgroundAttachment backgroundBlendMode backgroundClip " +
    "backgroundColor backgroundImage backgroundOrigin backgroundPosition backgroundPositionX backgroundPositionY " +
    "backgroundRepeat backgroundSize baselineShift baselineSource basePalette blockSize border borderBlock " +
    "borderBlockColor borderBlockEnd borderBlockEndColor borderBlockEndStyle borderBlockEndWidth borderBlockStart " +
    "borderBlockStartColor borderBlockStartStyle borderBlockStartWidth borderBlockStyle borderBlockWidth " +
    "borderBottom borderBottomColor borderBottomLeftRadius borderBottomRightRadius borderBottomStyle " +
    "borderBottomWidth borderCollapse borderColor borderEndEndRadius borderEndStartRadius borderImage " +
    "borderImageOutset borderImageRepeat borderImageSlice borderImageSource borderImageWidth borderInline " +
    "borderInlineColor borderInlineEnd borderInlineEndColor borderInlineEndStyle borderInlineEndWidth " +
    "borderInlineStart borderInlineStartColor borderInlineStartStyle borderInlineStartWidth borderInlineStyle " +
    "borderInlineWidth borderLeft borderLeftColor borderLeftStyle borderLeftWidth borderRadius borderRight " +
    "borderRightColor borderRightStyle borderRightWidth borderShape borderSpacing borderStartEndRadius " +
    "borderStartStartRadius borderStyle borderTop borderTopColor borderTopLeftRadius borderTopRightRadius " +
    "borderTopStyle borderTopWidth borderWidth bottom boxDecorationBreak boxShadow boxSizing breakAfter " +
    "breakBefore breakInside bufferedRendering captionSide caretAnimation caretColor caretShape clear clip " +
    "clipPath clipRule color colorInterpolation colorInterpolationFilters colorRendering colorScheme columnCount " +
    "columnFill columnGap columnHeight columnRule columnRuleColor columnRuleStyle columnRuleWidth columns " +
    "columnSpan columnWidth columnWrap contain container containerName containerType containIntrinsicBlockSize " +
    "containIntrinsicHeight containIntrinsicInlineSize containIntrinsicSize containIntrinsicWidth content " +
    "contentVisibility cornerBlockEndShape cornerBlockStartShape cornerBottomLeftShape cornerBottomRightShape " +
    "cornerBottomShape cornerEndEndShape cornerEndStartShape cornerInlineEndShape cornerInlineStartShape " +
    "cornerLeftShape cornerRightShape cornerShape cornerStartEndShape cornerStartStartShape cornerTopLeftShape " +
    "cornerTopRightShape cornerTopShape counterIncrement counterReset counterSet cssFloat cursor cx cy d " +
    "descentOverride direction display dominantBaseline dynamicRangeLimit emptyCells fallback fieldSizing fill " +
    "fillOpacity fillRule filter flex flexBasis flexDirection flexFlow flexGrow flexShrink flexWrap float " +
    "floodColor floodOpacity font fontDisplay fontFamily fontFeatureSettings fontKerning fontLanguageOverride " +
    "fontOpticalSizing fontPalette fontSize fontSizeAdjust fontStretch fontStyle fontSynthesis " +
    "fontSynthesisSmallCaps fontSynthesisStyle fontSynthesisWeight fontVariant fontVariantAlternates " +
    "fontVariantCaps fontVariantEastAsian fontVariantEmoji fontVariantLigatures fontVariantNumeric " +
    "fontVariantPosition fontVariationSettings fontWeight forcedColorAdjust gap grid gridArea gridAutoColumns " +
    "gridAutoFlow gridAutoRows gridColumn gridColumnEnd gridColumnGap gridColumnStart gridGap gridRow gridRowEnd " +
    "gridRowGap gridRowStart gridTemplate gridTemplateAreas gridTemplateColumns gridTemplateRows height " +
    "hyphenateCharacter hyphenateLimitChars hyphens imageOrientation imageRendering inherits initialLetter " +
    "initialValue inlineSize inset insetBlock insetBlockEnd insetBlockStart insetInline insetInlineEnd " +
    "insetInlineStart interactivity interestDelay interestDelayEnd interestDelayStart interpolateSize isolation " +
    "justifyContent justifyItems justifySelf left letterSpacing lightingColor lineBreak lineGapOverride lineHeight " +
    "listStyle listStyleImage listStylePosition listStyleType margin marginBlock marginBlockEnd marginBlockStart " +
    "marginBottom marginInline marginInlineEnd marginInlineStart marginLeft marginRight marginTop marker markerEnd " +
    "markerMid markerStart mask maskClip maskComposite maskImage maskMode maskOrigin maskPosition maskRepeat " +
    "maskSize maskType mathDepth mathShift mathStyle maxBlockSize maxHeight maxInlineSize maxWidth minBlockSize " +
    "minHeight minInlineSize minWidth mixBlendMode navigation negative objectFit objectPosition objectViewBox " +
    "offset offsetAnchor offsetDistance offsetPath offsetPosition offsetRotate opacity order orphans outline " +
    "outlineColor outlineOffset outlineStyle outlineWidth overflow overflowAnchor overflowBlock overflowClipMargin " +
    "overflowInline overflowWrap overflowX overflowY overlay overrideColors overscrollBehavior " +
    "overscrollBehaviorBlock overscrollBehaviorInline overscrollBehaviorX overscrollBehaviorY pad padding " +
    "paddingBlock paddingBlockEnd paddingBlockStart paddingBottom paddingInline paddingInlineEnd " +
    "paddingInlineStart paddingLeft paddingRight paddingTop page pageBreakAfter pageBreakBefore pageBreakInside " +
    "pageOrientation paintOrder perspective perspectiveOrigin placeContent placeItems placeSelf pointerEvents " +
    "position positionAnchor positionArea positionTry positionTryFallbacks positionTryOrder positionVisibility " +
    "prefix printColorAdjust quotes r range readingFlow readingOrder resize result right rotate rowGap rubyAlign " +
    "rubyPosition rx ry scale scrollbarColor scrollbarGutter scrollbarWidth scrollBehavior scrollInitialTarget " +
    "scrollMargin scrollMarginBlock scrollMarginBlockEnd scrollMarginBlockStart scrollMarginBottom " +
    "scrollMarginInline scrollMarginInlineEnd scrollMarginInlineStart scrollMarginLeft scrollMarginRight " +
    "scrollMarginTop scrollMarkerGroup scrollPadding scrollPaddingBlock scrollPaddingBlockEnd " +
    "scrollPaddingBlockStart scrollPaddingBottom scrollPaddingInline scrollPaddingInlineEnd " +
    "scrollPaddingInlineStart scrollPaddingLeft scrollPaddingRight scrollPaddingTop scrollSnapAlign scrollSnapStop " +
    "scrollSnapType scrollTargetGroup scrollTimeline scrollTimelineAxis scrollTimelineName shapeImageThreshold " +
    "shapeMargin shapeOutside shapeRendering size sizeAdjust speak speakAs src stopColor stopOpacity stroke " +
    "strokeDasharray strokeDashoffset strokeLinecap strokeLinejoin strokeMiterlimit strokeOpacity strokeWidth " +
    "suffix symbols syntax system tableLayout tabSize textAlign textAlignLast textAnchor textAutospace textBox " +
    "textBoxEdge textBoxTrim textCombineUpright textDecoration textDecorationColor textDecorationLine " +
    "textDecorationSkipInk textDecorationStyle textDecorationThickness textEmphasis textEmphasisColor " +
    "textEmphasisPosition textEmphasisStyle textIndent textJustify textOrientation textOverflow textRendering " +
    "textShadow textSizeAdjust textSpacingTrim textTransform textUnderlineOffset textUnderlinePosition textWrap " +
    "textWrapMode textWrapStyle timelineScope timelineTrigger timelineTriggerActivationRange " +
    "timelineTriggerActivationRangeEnd timelineTriggerActivationRangeStart timelineTriggerActiveRange " +
    "timelineTriggerActiveRangeEnd timelineTriggerActiveRangeStart timelineTriggerName timelineTriggerSource top " +
    "touchAction transform transformBox transformOrigin transformStyle transition transitionBehavior " +
    "transitionDelay transitionDuration transitionProperty transitionTimingFunction translate triggerScope types " +
    "unicodeBidi unicodeRange userSelect vectorEffect verticalAlign viewTimeline viewTimelineAxis " +
    "viewTimelineInset viewTimelineName viewTransitionClass viewTransitionGroup viewTransitionName " +
    "viewTransitionScope visibility whiteSpace whiteSpaceCollapse widows width willChange wordBreak wordSpacing " +
    "wordWrap writingMode x y zIndex zoom"
  ).split(" "),
);

/** Whether a style declaration has the property `name` (`name in el.style`), as in Chrome. */
export function isStyleName(name: string): boolean {
  return NAMES.has(name) || name.startsWith("--") || /^webkit[A-Z]/.test(name);
}
