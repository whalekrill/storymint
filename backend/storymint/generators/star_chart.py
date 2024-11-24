import math
from io import BytesIO

from PIL import Image, ImageDraw, ImageFont


class StarChart:
    """Star chart."""

    def __init__(self, attributes: dict, size: int = 400) -> None:
        """Initialize."""
        self.attributes = attributes
        self.size = size
        self.center = self.size / 2

    def generate(self) -> BytesIO:
        """Generate star chart."""
        image = Image.new("RGBA", (self.size, self.size), (26, 26, 26, 255))
        buffer = BytesIO()
        self.create_star_chart(image)
        image.save(buffer, format="PNG")
        return buffer

    def create_star_chart(self, image: Image) -> None:
        """Creates a star chart with letter labels and a legend."""
        # Create image with 4x resolution for anti-aliasing
        large_size = self.size * 4
        large_image = Image.new("RGBA", (large_size, large_size), (26, 26, 26, 255))
        draw = ImageDraw.Draw(large_image)
        # Scale up all coordinates
        orig_center = self.center
        self.center = self.center * 4
        self.size = large_size

        # Move star left and up to make room for legend
        # Apply offset to center
        center_x = self.center * 0.85
        center_y = self.center * 0.95

        # Create and transform the boundary star
        star_radius = self.size * 0.6  # Doubled from 0.32 to make star much larger
        boundary_points = []
        inner_radius = star_radius * 0.4

        # Generate star points directly
        for i in range(5):
            # Outer point
            outer_angle = (-90 + (i * 72)) * math.pi / 180
            boundary_points.append(
                (
                    center_x + star_radius * math.cos(outer_angle),
                    center_y + star_radius * math.sin(outer_angle),
                )
            )
            # Inner point
            inner_angle = (-90 + 36 + (i * 72)) * math.pi / 180
            boundary_points.append(
                (
                    center_x + inner_radius * math.cos(inner_angle),
                    center_y + inner_radius * math.sin(inner_angle),
                )
            )

        boundary_points = self.apply_transform(boundary_points)

        # Generate value points directly
        max_radius = self.size * 0.6  # Match the boundary star size
        min_radius = self.size * 0.25  # Adjusted for larger star
        value_points = []
        avg_value = sum(self.attributes.values()) / len(self.attributes)
        base_radius = min_radius + (max_radius - min_radius) * avg_value * 0.5

        for i, value in enumerate(self.attributes.values()):
            # Outer point
            angle = (-90 + (i * 72)) * math.pi / 180
            radius = base_radius + (max_radius - base_radius) * value
            value_points.append(
                (
                    center_x + radius * math.cos(angle),
                    center_y + radius * math.sin(angle),
                )
            )
            # Inner point
            inner_angle = (-90 + 36 + (i * 72)) * math.pi / 180
            inner_radius = base_radius * 0.4
            value_points.append(
                (
                    center_x + inner_radius * math.cos(inner_angle),
                    center_y + inner_radius * math.sin(inner_angle),
                )
            )

        value_points = self.apply_transform(value_points)

        # Draw boundary star with thicker lines
        for i in range(len(boundary_points)):
            start = boundary_points[i]
            end = boundary_points[(i + 1) % len(boundary_points)]
            draw.line([start, end], fill=(74, 74, 74, 255), width=8)

        # Draw value star
        # Fill
        draw.polygon(value_points, fill=(255, 165, 0, 64))
        # Outline
        for i in range(len(value_points)):
            start = value_points[i]
            end = value_points[(i + 1) % len(value_points)]
            draw.line([start, end], fill=(255, 165, 0, 255), width=8)

        try:
            font = ImageFont.truetype("DejaVuSans.ttf", size=56)  # 4x larger font
            small_font = ImageFont.truetype("DejaVuSans.ttf", size=40)  # For letters
        except Exception:
            font = ImageFont.load_default()
            small_font = font

        # Draw points at vertices with letter labels
        letters = ["A", "B", "C", "D", "E"]
        outer_points = value_points[::2]  # Only outer points
        legend_items = list(zip(letters, self.attributes.items(), strict=False))

        # Consistent label distance
        label_distance = 50  # Distance from point to label

        # Custom label positioning for each point
        label_positions = [
            (0, -1),  # A: above
            (1, 0),  # B: right
            (0, 1),  # C: below
            (-0.5, 1),  # D: below and left
            (-0.5, -1),  # E: above and left
        ]

        for point, letter, position in zip(
            outer_points, letters, label_positions, strict=False
        ):
            # Draw point
            draw.ellipse(
                [point[0] - 16, point[1] - 16, point[0] + 16, point[1] + 16],
                fill=(255, 165, 0, 255),
            )

            # Calculate label position
            dx, dy = position
            try:
                bbox = draw.textbbox((0, 0), letter, font=small_font)
                text_width = bbox[2] - bbox[0]
                text_height = bbox[3] - bbox[1]
            except Exception:
                text_width, text_height = draw.textsize(letter, font=small_font)

            # Position label with consistent distance but in different directions
            label_x = point[0] + (dx * label_distance) - text_width / 2
            label_y = point[1] + (dy * label_distance) - text_height / 2

            draw.text(
                (label_x, label_y),
                letter,
                fill=(255, 215, 0, 255),
                font=small_font,
            )

        # Draw legend in top left
        legend_start_x = 40
        legend_start_y = 40
        line_height = 70

        for i, (letter, (key, value)) in enumerate(legend_items):
            text = f"{letter}: {key} ({int(value * 5)})"
            draw.text(
                (legend_start_x, legend_start_y + i * line_height),
                text,
                fill=(255, 215, 0, 255),
                font=font,
            )

        # Reset size and center
        self.center = orig_center
        self.size = self.size // 4
        # Resize back down with anti-aliasing
        image.paste(large_image.resize((self.size, self.size), Image.LANCZOS))

    def create_star_points(self, radius: int, inner_radius_ratio: float = 0.4) -> list:
        """Creates the points for a star with the given radius."""
        points = []
        inner_radius = radius * inner_radius_ratio
        for i in range(5):
            # Outer point
            outer_angle = (-90 + (i * 72)) * math.pi / 180
            points.append(
                (
                    self.center + radius * math.cos(outer_angle),
                    self.center + radius * math.sin(outer_angle),
                )
            )
            # Inner point
            inner_angle = (-90 + 36 + (i * 72)) * math.pi / 180
            points.append(
                (
                    self.center + inner_radius * math.cos(inner_angle),
                    self.center + inner_radius * math.sin(inner_angle),
                )
            )
        return points

    def generate_value_points(self, attributes: dict) -> list:
        """Generates the points for the star based on the attributes."""
        max_radius = self.size * 0.35
        min_radius = self.size * 0.15
        points = []
        # Calculate average value for base size
        avg_value = sum(attributes.values()) / len(attributes)
        base_radius = min_radius + (max_radius - min_radius) * avg_value * 0.5
        for i, value in enumerate(attributes.values()):
            # Outer point
            angle = (-90 + (i * 72)) * math.pi / 180
            radius = base_radius + (max_radius - base_radius) * value
            points.append(
                (
                    self.center + radius * math.cos(angle),
                    self.center + radius * math.sin(angle),
                )
            )
            # Inner point
            inner_angle = (-90 + 36 + (i * 72)) * math.pi / 180
            inner_radius = base_radius * 0.4
            points.append(
                (
                    self.center + inner_radius * math.cos(inner_angle),
                    self.center + inner_radius * math.sin(inner_angle),
                )
            )
        return points

    def apply_transform(self, points: list) -> list:
        """Applies a series of transformations to the points."""
        transformed_points = []
        for x, y in points:
            # Center the point
            x -= self.center
            y -= self.center

            # Apply rotation (10 degrees)
            angle = 10 * math.pi / 180
            x_rot = x * math.cos(angle) - y * math.sin(angle)
            y_rot = x * math.sin(angle) + y * math.cos(angle)

            # Apply skew (-10deg, 0deg)
            skew_x = -10 * math.pi / 180
            x_skew = x_rot + y_rot * math.tan(skew_x)
            y_skew = y_rot

            # Apply scale (0.9, 0.7)
            x_scale = x_skew * 0.9
            y_scale = y_skew * 0.7

            # Move back from center
            transformed_points.append((x_scale + self.center, y_scale + self.center))
        return transformed_points
