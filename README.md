# Borders add-on for Cut Craft

Generates one repeating length of picture-frame moulding (Classical Baroque, Art Deco,
Gothic Pewter and six more styles, or your own description) with Google's image model,
then turns that picture into a depth map. Cut Craft saves both in your Library under
**Generated > Borders**, ready to place in a drawing like any 3D model.

## Install

In Cut Craft: **Gadgets > Add-ons > Borders**. Then open **Generate Border**.

## Your Google key

The add-on needs a Google AI Studio API key of your own (https://aistudio.google.com/apikey).
Type it into the Generate Border form's **Google key** box: Cut Craft keeps it in your
computer's password store and hands it to this program only while it runs. No key ships
with the add-on, and none is stored in this repository. The pictures you generate are
billed to your own Google account.

## What the program does

Cut Craft runs it; you never need to.

```text
cutcraft-borders picture STYLE RATIO DETAILS OUT   one repeat of a moulding, as a picture
cutcraft-borders depth PICTURE RATIO OUT           that picture's depth map (white highest)
cutcraft-borders styles                            the styles, one "id<TAB>name" a line
```

The key comes in the `CUTCRAFT_GOOGLE_API_KEY` environment variable. `CUTCRAFT_GOOGLE_IMAGE_MODEL`
picks another image model.

## Licence

PolyForm Noncommercial 1.0.0: see [LICENSE](LICENSE).
