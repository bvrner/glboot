use gl::types::*;
use std::convert::TryInto;

#[derive(Debug)]
pub struct Framebuffer {
    fbo: GLuint,
    texture: GLuint,
    rbo: GLuint,
    width: GLsizei,
    height: GLsizei,
}

impl Framebuffer {
    pub fn new(width: GLsizei, height: GLsizei) -> Result<Self, String> {
        unsafe {
            let mut id = 0;
            gl::GenFramebuffers(1, &mut id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);

            let mut texture = 0;
            gl::GenTextures(1, &mut texture);

            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB.try_into().unwrap(),
                width,
                height,
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
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture,
                0,
            );

            let mut renderbuffer = 0;
            gl::GenRenderbuffers(1, &mut renderbuffer);
            gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffer);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                renderbuffer,
            );

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::DeleteFramebuffers(1, &id);
                gl::DeleteTextures(1, &texture);
                gl::DeleteRenderbuffers(1, &renderbuffer);

                Err(String::from("Framebuffer error"))
            } else {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

                Ok(Framebuffer {
                    fbo: id,
                    texture,
                    rbo: renderbuffer,
                    width,
                    height,
                })
            }
        }
    }

    pub fn new_multisampled(
        width: GLsizei,
        height: GLsizei,
        samples: GLsizei,
    ) -> Result<Self, String> {
        unsafe {
            let mut id = 0;
            gl::GenFramebuffers(1, &mut id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);

            let mut texture = 0;
            gl::GenTextures(1, &mut texture);

            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, texture);
            gl::TexImage2DMultisample(
                gl::TEXTURE_2D_MULTISAMPLE,
                samples,
                gl::RGB.try_into().unwrap(),
                width,
                height,
                gl::TRUE,
            );
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, 0);

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D_MULTISAMPLE,
                texture,
                0,
            );

            let mut renderbuffer = 0;
            gl::GenRenderbuffers(1, &mut renderbuffer);
            gl::BindRenderbuffer(gl::RENDERBUFFER, renderbuffer);
            gl::RenderbufferStorageMultisample(
                gl::RENDERBUFFER,
                samples,
                gl::DEPTH24_STENCIL8,
                width,
                height,
            );
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);

            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                renderbuffer,
            );

            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                gl::DeleteFramebuffers(1, &id);
                gl::DeleteTextures(1, &texture);
                gl::DeleteRenderbuffers(1, &renderbuffer);

                Err(String::from("Framebuffer error"))
            } else {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

                Ok(Framebuffer {
                    fbo: id,
                    texture,
                    rbo: renderbuffer,
                    width,
                    height,
                })
            }
        }
    }

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
                gl::LINEAR,
            );

            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        }
    }

    pub fn update_dimensions(&mut self, width: GLsizei, height: GLsizei) {
        self.width = width;
        self.height = height;
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB.try_into().unwrap(),
                width,
                height,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindRenderbuffer(gl::RENDERBUFFER, self.rbo);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        }
    }

    #[inline]
    pub fn bind_texture(&self, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
        }
    }

    #[inline]
    pub fn unbind_texture(&self) {
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
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteRenderbuffers(1, &self.rbo);
        }
    }
}
