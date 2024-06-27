pub fn trace_ray<const N: usize>(
    &self,
    rng: &mut RandGen,
    scene: &Scene,
    mut ray: Ray3D,
    aux: &mut [usize; N],
) -> Spectrum {
    // Keep a copy of the original ray, so we
    // can start over for every ambient sample.
    let original_ray = ray;
    // It is dark by default
    let mut spectrum = Spectrum::BLACK;
    // If max_depth is Zero, then there is no point in
    // using multiple ambient samples
    let n_ambient_samples = if self.max_depth == 0 {
        1
    } else {
        self.n_ambient_samples
    };

    // For each ambient sample
    for _ in 0..n_ambient_samples {
        // Reset the ray
        ray = original_ray;
        // Reset the throughput to 1
        let mut beta = Spectrum::ONE;
        // Reset depth to 0
        let mut depth = 0;

        let mut specular_bounce = true;
        loop {
            // Bouncing loop
            let intersect = scene.cast_ray(ray, aux);
            if intersect.is_none() {
                // No hit.
                break;
            }

            let (triangle_index, mut interaction) = intersect.unwrap();
            // Front and back materials are supported... not sure if a good idea yet
            let material = match interaction.geometry_shading.side {
                SurfaceSide::Front => {
                    &scene.materials[scene.front_material_indexes[triangle_index]]
                }
                SurfaceSide::Back => &scene.materials[scene.back_material_indexes[triangle_index]],
                SurfaceSide::NonApplicable => {
                    // Hit parallel to the surface...
                    break;
                }
            };

            // We hit a light... lights do not reflect,
            // so break
            if material.emits_light() {
                if specular_bounce {
                    spectrum += beta * material.colour();
                }
                break;
            }

            interaction.interpolate_normal(scene.normals[triangle_index]);

            // Direct lighting
            let n_shadow_samples = if depth == 0 { self.n_shadow_samples } else { 1 };
            let local = self.get_local_illumination(
                scene,
                material,
                &interaction,
                refraction_coefficient,
                rng,
                n_shadow_samples,
                aux,
            );
            spectrum += beta * local;

            // reached limit.
            depth += 1;
            if depth > self.max_depth {
                break;
            }

            // Spawn a ray in a new direction
            let u = rng.gen(); // get random value
            let new_dir = sample_uniform_hemisphere(u); // create a new upward-looking direction
            let cos_theta = new_dir.z; // cosine of the angle between UP and new_dir
            let pdf = 0.5 / PI; // uniformly distributed, so 1/2*PI
            let spectrum = material.colour() / PI; // rho over P for lambertian materials

            /****************************/
            // THIS 1.07 IS THE HACK
            beta *= spectrum * cos_theta / (pdf * 1.07);
            /****************************/
            specular_bounce = false;
            let (_, normal, e1, e2) = interaction.get_triad();
            ray = Ray3D {
                direction: material.to_world(normal, e1, e2, new_dir), // appropriately transform the local ray into worlds coordinates
                origin: interaction.point + normal * 0.001,
            }
        }
    }

    spectrum / n_ambient_samples as Float
}
