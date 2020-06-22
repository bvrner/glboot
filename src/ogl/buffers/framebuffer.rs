use gl::types::*;
use std::convert::TryInto;

#[derive(Debug, Copy, Clone)]
pub struct FramebufferBuilder {
    width: GLsizei,
    height: GLsizei,
    color_binds: u32,
    depth: bool,
    stencil: bool,
    multisample: Option<GLsizei>,
}

impl FramebufferBuilder {
    pub fn new(width: GLsizei, height: GLsizei) -> Self {
        FramebufferBuilder {
            width,
            height,
            color_binds: 1,
            depth: false,
            stencil: false,
            multisample: None,
        }
    }

    pub fn with_depth_and_stencil(&mut self) -> &mut Self {
        self.depth = true;
        self.stencil = true;
        self
    }

    pub fn with_stencil(&mut self) -> &mut Self {
        self.stencil = true;
        self
    }

    pub fn with_depth(&mut self) -> &mut Self {
        self.depth = true;
        self
    }

    pub fn color_attach(&mut self, color_binds: u32) -> &mut Self {
        self.color_binds = color_binds;
        self
    }

    pub fn with_samples(&mut self, samples: GLsizei) -> &mut Self {
        self.multisample = Some(samples);
        self
    }

    pub fn build(self) -> Result<Framebuffer, String> {
        let mut fbo = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        }

        let textures: Vec<GLuint> = (0..self.color_binds)
            .map(|attach_point| {
                let mut texture = 0;
                unsafe {
                    gl::GenTextures(1, &mut texture);
                    gl::BindTexture(
                        if self.multisample.is_some() {
                            gl::TEXTURE_2D_MULTISAMPLE
                        } else {
                            gl::TEXTURE_2D
                        },
                        texture,
                    );
                    if let Some(samples) = self.multisample {
                        gl::TexImage2DMultisample(
                            gl::TEXTURE_2D_MULTISAMPLE,
                            samples,
                            gl::RGB.try_into().unwrap(),
                            self.width,
                            self.height,
                            gl::TRUE,
                        );
                        gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, 0);

                        gl::FramebufferTexture2D(
                            gl::FRAMEBUFFER,
                            gl::COLOR_ATTACHMENT0 + attach_point,
                            gl::TEXTURE_2D_MULTISAMPLE,
                            texture,
                            0,
                        );
                    } else {
                        gl::TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::RGB.try_into().unwrap(),
                            self.width,
                            self.height,
                            0,
                            gl::RGB,
                            gl::UNSIGNED_BYTE,
                            std::ptr::null(),
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_MIN_FILTER,
                            gl::LINEAR.try_into().unwrap(),
                        );
                        gl::TexParameteri(
                            gl::TEXTURE_2D,
                            gl::TEXTURE_MAG_FILTER,
                            gl::LINEAR.try_into().unwrap(),
                        );
                        gl::BindTexture(gl::TEXTURE_2D, 0);

                        gl::FramebufferTexture2D(
                            gl::FRAMEBUFFER,
                            gl::COLOR_ATTACHMENT0 + attach_point,
                            gl::TEXTURE_2D,
                            texture,
                            0,
                        );
                    }
                }
                texture
            })
            .collect();

        let mut rbo = 0;
        if self.stencil || self.depth {
            unsafe {
                let rbo_format = if self.depth && self.stencil {
                    gl::DEPTH24_STENCIL8
                } else {
                    unimplemented!()
                };
                gl::GenRenderbuffers(1, &mut rbo);
                gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
                gl::RenderbufferStorageMultisample(
                    gl::RENDERBUFFER,
                    self.multisample.unwrap_or(0),
                    rbo_format,
                    self.width,
                    self.height,
                );
                gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
                gl::FramebufferRenderbuffer(
                    gl::FRAMEBUFFER,
                    gl::DEPTH_STENCIL_ATTACHMENT,
                    gl::RENDERBUFFER,
                    rbo,
                );
            }
        }
        unsafe {
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::DeleteFramebuffers(1, &fbo);
                gl::DeleteTextures(textures.len() as i32, textures.as_ptr());
                if rbo != 0 {
                    gl::DeleteRenderbuffers(1, &rbo)
                };

                Err(String::from("Framebuffer error"))
            } else {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

                Ok(Framebuffer {
                    fbo,
                    textures,
                    rbo,
                    width: self.width,
                    height: self.height,
                    samples: self.multisample,
                })
            }
        }
    }
}

#[derive(Debug)]
pub struct Framebuffer {
    fbo: GLuint,
    textures: Vec<GLuint>,
    rbo: GLuint,
    width: GLsizei,
    height: GLsizei,
    samples: Option<GLsizei>,
}

impl Framebuffer {
    pub fn blit(&self, other: &Self) {
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, other.fbo);

            gl::BlitFramebuffer(
                0,
                0,
                self.width,
                self.height,
                0,
                0,
                other.width,
                other.height,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );

            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        }
    }

    // pub fn update_dimensions(&mut self, width: GLsizei, height: GLsizei) {
    //     self.width = width;
    //     self.height = height;
    //     unsafe {
    //         if let Some(samples) = self.samples {
    //             gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, self.texture);
    //             gl::TexImage2DMultisample(
    //                 gl::TEXTURE_2D_MULTISAMPLE,
    //                 samples,
    //                 gl::RGB.try_into().unwrap(),
    //                 width,
    //                 height,
    //                 gl::TRUE,
    //             );
    //             gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, 0);
    //             gl::BindRenderbuffer(gl::RENDERBUFFER, self.rbo);
    //             gl::RenderbufferStorageMultisample(
    //                 gl::RENDERBUFFER,
    //                 samples,
    //                 gl::DEPTH24_STENCIL8,
    //                 width,
    //                 height,
    //             );
    //             gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
    //         } else {
    //             gl::BindTexture(gl::TEXTURE_2D, self.texture);
    //             gl::TexImage2D(
    //                 gl::TEXTURE_2D,
    //                 0,
    //                 gl::RGB.try_into().unwrap(),
    //                 width,
    //                 height,
    //                 0,
    //                 gl::RGB,
    //                 gl::UNSIGNED_BYTE,
    //                 std::ptr::null(),
    //             );
    //             gl::BindTexture(gl::TEXTURE_2D, 0);
    //             gl::BindRenderbuffer(gl::RENDERBUFFER, self.rbo);
    //             gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
    //             gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
    //         }
    //     }
    // }

    /// Binds the color attachment textures from this framebuffer in the active slots from `slot` ownards.
    #[inline]
    pub fn bind_textures(&self, slot: u32) {
        for (i, &texture) in self.textures.iter().enumerate() {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + slot + i as u32);
                gl::BindTexture(gl::TEXTURE_2D, texture);
            }
        }
    }

    #[inline]
    pub fn unbind_textures(&self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) }
    }

    #[inline]
    pub fn bind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo) }
    }

    #[inline]
    pub fn unbind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(self.textures.len() as i32, self.textures.as_ptr());
            if self.rbo != 0 {
                gl::DeleteRenderbuffers(1, &self.rbo);
            }
            gl::DeleteFramebuffers(1, &self.fbo);
        }
    }
}
