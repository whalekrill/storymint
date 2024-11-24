import io
import math

from django.core.files.base import ContentFile
from Pillow import Image, ImageDraw


class StarChart:
    """Star chart."""

    def __call__(self, attributes: dict, size: int = 400) -> None:
        """Creates a star chart with the given attributes."""
        self.size = size
        self.center = self.size / 2
        # Save to buffer
        # Create image with dark background
        image = Image.new("RGBA", (self.size, self.size), (26, 26, 26, 255))
        self.create_star_image(image, attributes)
        buffer = io.BytesIO()
        image.save(buffer, format="PNG")
        return ContentFile(buffer.getvalue())

    def create_star_image(self, image: Image, attributes: dict) -> None:
        """Creates a star chart."""
        draw = ImageDraw.Draw(image)
        # Create and transform the boundary star
        boundary_points = self.create_star_points(self.size * 0.35)
        boundary_points = self.apply_transform(boundary_points)

        # Create and transform the value star
        value_points = self.generate_value_points(attributes)
        value_points = self.apply_transform(value_points)

        # Draw boundary star
        for i in range(len(boundary_points)):
            start = boundary_points[i]
            end = boundary_points[(i + 1) % len(boundary_points)]
            draw.line([start, end], fill=(74, 74, 74, 255), width=2)

        # Draw value star
        # Fill
        draw.polygon(value_points, fill=(255, 165, 0, 64))
        # Outline
        for i in range(len(value_points)):
            start = value_points[i]
            end = value_points[(i + 1) % len(value_points)]
            draw.line([start, end], fill=(255, 165, 0, 255), width=2)

        # Draw points at vertices
        for point in value_points[::2]:  # Only outer points
            draw.ellipse(
                [point[0] - 4, point[1] - 4, point[0] + 4, point[1] + 4],
                fill=(255, 165, 0, 255),
            )

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
