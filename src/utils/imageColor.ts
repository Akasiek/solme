export function getDominantImageColor(image: HTMLImageElement): string | undefined {
  const width = 48;
  const height = 48;
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;

  const context = canvas.getContext("2d", { willReadFrequently: true });
  if (!context) {
    return undefined;
  }

  context.drawImage(image, 0, 0, width, height);

  const pixels = context.getImageData(0, 0, width, height).data;
  const buckets = new Map<string, { count: number; red: number; green: number; blue: number }>();

  for (let index = 0; index < pixels.length; index += 16) {
    const alpha = pixels[index + 3];
    if (alpha < 128) {
      continue;
    }

    const red = pixels[index];
    const green = pixels[index + 1];
    const blue = pixels[index + 2];
    const max = Math.max(red, green, blue);
    const min = Math.min(red, green, blue);
    const saturation = max === 0 ? 0 : (max - min) / max;
    const brightness = max / 255;

    if (saturation < 0.18 || brightness < 0.16 || brightness > 0.94) {
      continue;
    }

    const bucketRed = Math.round(red / 32) * 32;
    const bucketGreen = Math.round(green / 32) * 32;
    const bucketBlue = Math.round(blue / 32) * 32;
    const key = `${bucketRed}-${bucketGreen}-${bucketBlue}`;
    const bucket = buckets.get(key) ?? { count: 0, red: 0, green: 0, blue: 0 };
    const weight = 1 + saturation * 2;

    bucket.count += weight;
    bucket.red += red * weight;
    bucket.green += green * weight;
    bucket.blue += blue * weight;
    buckets.set(key, bucket);
  }

  let dominantBucket: { count: number; red: number; green: number; blue: number } | undefined;
  for (const bucket of buckets.values()) {
    if (!dominantBucket || bucket.count > dominantBucket.count) {
      dominantBucket = bucket;
    }
  }

  if (!dominantBucket) {
    return undefined;
  }

  const red = Math.round(dominantBucket.red / dominantBucket.count);
  const green = Math.round(dominantBucket.green / dominantBucket.count);
  const blue = Math.round(dominantBucket.blue / dominantBucket.count);

  return `${red} ${green} ${blue}`;
}
