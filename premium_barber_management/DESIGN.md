---
name: Premium Barber Management
colors:
  surface: '#121317'
  surface-dim: '#121317'
  surface-bright: '#38393d'
  surface-container-lowest: '#0d0e12'
  surface-container-low: '#1a1b1f'
  surface-container: '#1e1f23'
  surface-container-high: '#292a2e'
  surface-container-highest: '#343539'
  on-surface: '#e3e2e7'
  on-surface-variant: '#d0c5af'
  inverse-surface: '#e3e2e7'
  inverse-on-surface: '#2f3034'
  outline: '#99907c'
  outline-variant: '#4d4635'
  surface-tint: '#e9c349'
  primary: '#f2ca50'
  on-primary: '#3c2f00'
  primary-container: '#d4af37'
  on-primary-container: '#554300'
  inverse-primary: '#735c00'
  secondary: '#c8c6c5'
  on-secondary: '#313030'
  secondary-container: '#474746'
  on-secondary-container: '#b7b5b4'
  tertiary: '#d0cdcd'
  on-tertiary: '#303030'
  tertiary-container: '#b4b2b2'
  on-tertiary-container: '#454545'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#ffe088'
  primary-fixed-dim: '#e9c349'
  on-primary-fixed: '#241a00'
  on-primary-fixed-variant: '#574500'
  secondary-fixed: '#e5e2e1'
  secondary-fixed-dim: '#c8c6c5'
  on-secondary-fixed: '#1c1b1b'
  on-secondary-fixed-variant: '#474746'
  tertiary-fixed: '#e4e2e1'
  tertiary-fixed-dim: '#c8c6c5'
  on-tertiary-fixed: '#1b1c1c'
  on-tertiary-fixed-variant: '#474747'
  background: '#121317'
  on-background: '#e3e2e7'
  surface-variant: '#343539'
typography:
  display-lg:
    fontFamily: Libre Caslon Text
    fontSize: 48px
    fontWeight: '700'
    lineHeight: 56px
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Libre Caslon Text
    fontSize: 32px
    fontWeight: '600'
    lineHeight: 40px
  headline-md:
    fontFamily: Libre Caslon Text
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
  body-lg:
    fontFamily: Hanken Grotesk
    fontSize: 18px
    fontWeight: '400'
    lineHeight: 28px
  body-md:
    fontFamily: Hanken Grotesk
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  label-md:
    fontFamily: Hanken Grotesk
    fontSize: 14px
    fontWeight: '600'
    lineHeight: 20px
    letterSpacing: 0.05em
  label-sm:
    fontFamily: Hanken Grotesk
    fontSize: 12px
    fontWeight: '500'
    lineHeight: 16px
  headline-lg-mobile:
    fontFamily: Libre Caslon Text
    fontSize: 28px
    fontWeight: '600'
    lineHeight: 36px
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  unit: 4px
  gutter: 24px
  margin-desktop: 40px
  margin-mobile: 16px
  container-max: 1440px
---

## Brand & Style

This design system is built for a high-end, masculine grooming environment. The brand personality is authoritative, meticulous, and sophisticated, mirroring the precision of a master barber. The target audience consists of shop owners and elite staff who require a tool that feels as premium as the services they provide.

The visual style is **Minimalist with Tactile accents**. It leverages deep, monochromatic foundations to create a sense of exclusivity and "members-only" luxury. The interface avoids unnecessary flair, focusing instead on high-quality typography and precise alignment. The emotional response should be one of calm control and professional excellence, achieved through generous whitespace and a "dark mode by default" aesthetic that reduces eye strain during long shifts.

## Colors

The palette is rooted in a "Luxury Dark" philosophy. The primary color is a refined **Gold (#D4AF37)**, used sparingly for critical actions, active states, and brand highlights. An **Amber (#FFBF00)** variant is reserved for interactive hover states to provide a warm, glowing feedback loop.

The foundation uses a tiered black system:
- **Background Deep (#0F0F0F):** The primary canvas color.
- **Secondary Surface (#1A1A1A):** Used for sidebars and navigation containers.
- **Tertiary Surface (#2C2C2C):** Used for elevated cards and modals.
- **Border Subtle (#333333):** Defines structure without breaking the dark immersion.

Text is primarily Off-White or Light Slate to ensure high legibility against the charcoal backgrounds without the harshness of pure white.

## Typography

This design system utilizes a high-contrast typographic pairing to balance tradition with modernity. 

**Libre Caslon Text** is used for headlines and display elements. Its sharp serifs and classical proportions evoke the heritage of traditional barbering and editorial luxury. 

**Hanken Grotesk** serves as the primary interface typeface. It is a highly legible, contemporary sans-serif that ensures data-heavy scheduling views remain clear and easy to navigate. Labels and utility text use uppercase styling with increased letter spacing to create a structured, architectural feel.

## Layout & Spacing

The layout follows a **12-column Fluid Grid** for the main content area, anchored by a fixed-width left navigation bar (280px). To maintain a high-end feel, the system uses "Generous Density"—information is packed efficiently for management tasks, but separated by strict 24px gutters to prevent visual clutter.

- **Desktop:** 12 columns, 24px gutters, 40px outer margins.
- **Tablet:** 8 columns, 16px gutters, 24px outer margins.
- **Mobile:** 4 columns, 12px gutters, 16px outer margins.

The spacing rhythm is based on a 4px baseline grid. Internal component padding should favor vertical breathing room to reinforce the sophisticated aesthetic.

## Elevation & Depth

In this design system, depth is communicated through **Tonal Layering** and **Subtle Outlines** rather than heavy shadows. 

1.  **Base Level:** The deep charcoal background (#0F0F0F).
2.  **Container Level:** Cards and panels use #1A1A1A with a 1px border of #333333.
3.  **Floating Level:** Modals and dropdowns use #2C2C2C with a very soft, large-radius black shadow (0px 12px 32px rgba(0,0,0,0.5)) to separate them from the main UI.

Interactive elements like buttons or active menu items do not use depth, but rather a "glow" or "illumination" state using the Gold and Amber accents to signify they are "active" or "powered on."

## Shapes

The shape language is **Soft (0.25rem)**. This subtle rounding takes the edge off the dark, masculine palette, making the interface feel modern and engineered rather than aggressive. 

- **Small elements (Buttons, Inputs):** 4px radius.
- **Medium elements (Cards, Modals):** 8px radius.
- **Large containers:** 12px radius.

Buttons and selection chips should maintain these consistent radii; avoid pill-shaped or fully circular elements to preserve the "sharp" and "tailored" brand persona.

## Components

### Buttons
Primary buttons use a solid Gold (#D4AF37) fill with black text. Secondary buttons use a ghost style: a 1px border of Gold with Gold text. Tertiary buttons are text-only with uppercase labeling.

### Cards
Cards are the primary container for the dashboard. They must feature a subtle 1px border (#333333) and no shadow unless they are being dragged or hovered. The header of the card should use a small-cap label for the title.

### Input Fields
Inputs feature a dark background (#0F0F0F) and a subtle bottom border. Upon focus, the border transitions to Gold with a very faint amber outer glow.

### Scheduling Grid (Specialty Component)
The central management feature. Time slots should be demarcated by #333333 lines. Appointment blocks should use semi-transparent versions of the accent colors to allow the grid lines to remain visible, maintaining the "high-density" requirement.

### Chips & Status Indicators
Status indicators (e.g., "Confirmed," "In-Chair," "Completed") use small, circular dots of color next to Hanken Grotesk labels. Avoid large colorful backgrounds for chips to keep the UI clean.